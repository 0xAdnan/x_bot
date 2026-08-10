use crate::config::Config;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const COLD_FACTOR: f64 = 0.25;
const BURST_HR_LIMIT: usize = 10;
const MAX_CONSECUTIVE_TRIPS: usize = 3;
const WINDOW_SECS: u64 = 86400; // 24h

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub ts: String,
    pub action: String,
    pub handle: Option<String>,
    pub segment: Option<String>,
    pub variant: Option<String>,
    pub detail: Option<String>,
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetStatus {
    pub day: String,
    pub ramp_active: bool,
    pub ramp_until: String,
    pub caps: HashMap<String, usize>,
    pub used: HashMap<String, usize>,
    pub remaining: HashMap<String, usize>,
    pub actions_last_hour: usize,
    pub burst_limit: usize,
    pub burst_warning: bool,
}

pub fn get_skill_state_dir() -> PathBuf {
    let cfg = Config::load();
    cfg.repo_root
        .join(".opencode")
        .join("skills")
        .join("x-growth")
        .join("state")
}

pub fn circuit_breaker_status() -> (bool, String) {
    let state_dir = get_skill_state_dir();
    let hard_stop_file = state_dir.join("HARD_STOP");

    if hard_stop_file.exists() {
        let reason = fs::read_to_string(&hard_stop_file).unwrap_or_default();
        return (
            true,
            format!("PAUSED (state/HARD_STOP present). Reason: {}", reason.trim()),
        );
    }

    let trips = count_recent_trips();
    if trips >= MAX_CONSECUTIVE_TRIPS {
        return (
            true,
            format!(
                "PAUSED ({} consecutive trips in 24h, limit {}).",
                trips, MAX_CONSECUTIVE_TRIPS
            ),
        );
    }

    (false, format!("OK to run ({} trips in 24h).", trips))
}

pub fn circuit_breaker_trip(reason: &str) {
    let state_dir = get_skill_state_dir();
    let _ = fs::create_dir_all(&state_dir);
    let log_file = state_dir.join("circuit-breaker.jsonl");

    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let entry = serde_json::json!({
        "ts": now_ts,
        "reason": reason
    });

    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        let _ = writeln!(f, "{}", entry);
    }

    let trips = count_recent_trips();
    println!(
        "[Circuit Breaker] Trip recorded ({}/{} in 24h): {}",
        trips, MAX_CONSECUTIVE_TRIPS, reason
    );

    if trips >= MAX_CONSECUTIVE_TRIPS {
        let stop_file = state_dir.join("HARD_STOP");
        let msg = format!(
            "Circuit breaker tripped after {} consecutive trips in 24h. Last reason: {}",
            trips, reason
        );
        let _ = fs::write(stop_file, msg);
        println!("[Circuit Breaker] Created HARD_STOP file. Automation paused.");
    }
}

pub fn circuit_breaker_reset() {
    let state_dir = get_skill_state_dir();
    let stop_file = state_dir.join("HARD_STOP");
    let log_file = state_dir.join("circuit-breaker.jsonl");

    let _ = fs::remove_file(stop_file);
    let _ = fs::write(log_file, "");
    println!("[Circuit Breaker] Reset completed. Automation may resume.");
}

fn count_recent_trips() -> usize {
    let state_dir = get_skill_state_dir();
    let log_file = state_dir.join("circuit-breaker.jsonl");

    if !log_file.exists() {
        return 0;
    }

    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let mut count = 0;
    if let Ok(content) = fs::read_to_string(log_file) {
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(val) = serde_json::from_str::<Value>(line) {
                if let Some(ts) = val["ts"].as_u64() {
                    if now_ts.saturating_sub(ts) <= WINDOW_SECS {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

pub fn budget_check() -> BudgetStatus {
    let state_dir = get_skill_state_dir();
    let account_file = state_dir.join("account.json");
    let log_file = state_dir.join("activity-log.jsonl");

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let mut ramp_until = String::new();
    if account_file.exists() {
        if let Ok(content) = fs::read_to_string(account_file) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                if let Some(ru) = val["ramp_until"].as_str() {
                    ramp_until = ru.to_string();
                }
            }
        }
    }

    let ramp_active = !ramp_until.is_empty() && today < ramp_until;

    let default_caps: HashMap<&str, usize> = HashMap::from([
        ("like", 50),
        ("reply", 15),
        ("follow", 15),
        ("dm", 10),
        ("post", 4),
        ("quote", 4),
        ("discover", 40),
    ]);

    let mut caps = HashMap::new();
    for (k, v) in default_caps {
        let cap_val = if ramp_active {
            ((v as f64 * COLD_FACTOR) as usize).max(1)
        } else {
            v
        };
        caps.insert(k.to_string(), cap_val);
    }

    let mut used = HashMap::from([
        ("like".to_string(), 0),
        ("reply".to_string(), 0),
        ("follow".to_string(), 0),
        ("dm".to_string(), 0),
        ("post".to_string(), 0),
        ("quote".to_string(), 0),
        ("discover".to_string(), 0),
    ]);

    let mut actions_last_hour = 0;
    let now_epoch = chrono::Utc::now().timestamp();

    if log_file.exists() {
        if let Ok(content) = fs::read_to_string(log_file) {
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(e) = serde_json::from_str::<ActivityEntry>(line) {
                    if e.result.as_deref().unwrap_or("ok") != "ok" {
                        continue;
                    }
                    let day_str = e.ts.split('T').next().unwrap_or("");
                    if day_str != today {
                        continue;
                    }

                    let act = e.action.as_str();
                    if act == "dm" || act == "followup" {
                        *used.entry("dm".to_string()).or_insert(0) += 1;
                    } else if used.contains_key(act) {
                        *used.entry(act.to_string()).or_insert(0) += 1;
                    }

                    if let Ok(ts_parsed) = chrono::DateTime::parse_from_rfc3339(&e.ts) {
                        if now_epoch - ts_parsed.timestamp() <= 3600 {
                            actions_last_hour += 1;
                        }
                    }
                }
            }
        }
    }

    let mut remaining = HashMap::new();
    for (k, cap) in &caps {
        let u = used.get(k).copied().unwrap_or(0);
        remaining.insert(k.clone(), cap.saturating_sub(u));
    }

    let burst_warning = actions_last_hour >= BURST_HR_LIMIT;

    BudgetStatus {
        day: today,
        ramp_active,
        ramp_until,
        caps,
        used,
        remaining,
        actions_last_hour,
        burst_limit: BURST_HR_LIMIT,
        burst_warning,
    }
}
