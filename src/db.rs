use crate::config::Config;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentionJob {
    pub id: Option<i64>,
    pub tweet_id: String,
    pub user_handle: String,
    pub target_url: String,
    pub editor_job_id: Option<String>,
    pub status: String,
    pub s3_video_url: Option<String>,
    pub x_reply_id: Option<String>,
    pub tweet_text: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Prospect {
    pub id: Option<i64>,
    pub handle: String,
    pub name: Option<String>,
    pub url: Option<String>,
    pub segment: Option<String>,
    pub score: Option<i32>,
    pub stage: Option<String>,
    pub last_touch: Option<String>,
    pub next_action_date: Option<String>,
    pub touches: Option<i32>,
    pub product_url: Option<String>,
    pub last_variant: Option<String>,
    pub outcome: Option<String>,
    pub notes: Option<String>,
    pub why: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Activity {
    pub id: Option<i64>,
    pub ts: Option<String>,
    pub action: String,
    pub handle: Option<String>,
    pub segment: Option<String>,
    pub variant: Option<String>,
    pub detail: Option<String>,
    pub result: Option<String>,
}

pub struct Database {
    pub conn: Connection,
    pub repo_root: PathBuf,
}

impl Database {
    pub fn open() -> SqlResult<Self> {
        let cfg = Config::load();
        if let Some(parent) = cfg.db_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let conn = Connection::open(&cfg.db_path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS mention_jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tweet_id TEXT UNIQUE NOT NULL,
                user_handle TEXT NOT NULL,
                target_url TEXT NOT NULL,
                editor_job_id TEXT DEFAULT '',
                status TEXT NOT NULL DEFAULT 'pending',
                s3_video_url TEXT DEFAULT '',
                x_reply_id TEXT DEFAULT '',
                tweet_text TEXT DEFAULT '',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS prospects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                handle TEXT UNIQUE NOT NULL,
                name TEXT DEFAULT '',
                url TEXT DEFAULT '',
                segment TEXT DEFAULT 'founder',
                score INTEGER DEFAULT 0,
                stage TEXT DEFAULT 'new',
                last_touch TEXT,
                next_action_date TEXT,
                touches INTEGER DEFAULT 0,
                product_url TEXT DEFAULT '',
                last_variant TEXT DEFAULT '',
                outcome TEXT DEFAULT '',
                notes TEXT DEFAULT '',
                why TEXT DEFAULT '',
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS activities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts TEXT NOT NULL,
                action TEXT NOT NULL,
                handle TEXT DEFAULT '',
                segment TEXT DEFAULT '',
                variant TEXT DEFAULT '',
                detail TEXT DEFAULT '',
                result TEXT DEFAULT 'ok',
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS insights (
                id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1),
                content TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            ",
        )?;

        let _ = conn.execute_batch("ALTER TABLE mention_jobs ADD COLUMN tweet_text TEXT DEFAULT '';");

        Ok(Database {
            conn,
            repo_root: cfg.repo_root,
        })
    }

    pub fn upsert_mention_job(&self, job: &MentionJob) -> SqlResult<()> {
        self.conn.execute(
            "
            INSERT INTO mention_jobs (tweet_id, user_handle, target_url, editor_job_id, status, s3_video_url, x_reply_id, tweet_text, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)
            ON CONFLICT(tweet_id) DO UPDATE SET
              editor_job_id = COALESCE(NULLIF(excluded.editor_job_id, ''), mention_jobs.editor_job_id),
              status = excluded.status,
              s3_video_url = COALESCE(NULLIF(excluded.s3_video_url, ''), mention_jobs.s3_video_url),
              x_reply_id = COALESCE(NULLIF(excluded.x_reply_id, ''), mention_jobs.x_reply_id),
              tweet_text = COALESCE(NULLIF(excluded.tweet_text, ''), mention_jobs.tweet_text),
              updated_at = CURRENT_TIMESTAMP
            ",
            params![
                job.tweet_id,
                job.user_handle,
                job.target_url,
                job.editor_job_id.as_deref().unwrap_or(""),
                job.status,
                job.s3_video_url.as_deref().unwrap_or(""),
                job.x_reply_id.as_deref().unwrap_or(""),
                job.tweet_text.as_deref().unwrap_or(""),
            ],
        )?;
        Ok(())
    }

    pub fn get_mention_job_by_tweet_id(&self, tweet_id: &str) -> SqlResult<Option<MentionJob>> {
        let mut stmt = self.conn.prepare("SELECT id, tweet_id, user_handle, target_url, editor_job_id, status, s3_video_url, x_reply_id, created_at, updated_at, tweet_text FROM mention_jobs WHERE tweet_id = ?1")?;
        let mut rows = stmt.query([tweet_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(MentionJob {
                id: row.get(0)?,
                tweet_id: row.get(1)?,
                user_handle: row.get(2)?,
                target_url: row.get(3)?,
                editor_job_id: row.get(4)?,
                status: row.get(5)?,
                s3_video_url: row.get(6)?,
                x_reply_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                tweet_text: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_mention_jobs_by_status(&self, status: Option<&str>, limit: usize) -> SqlResult<Vec<MentionJob>> {
        let query = if status.is_some() {
            "SELECT id, tweet_id, user_handle, target_url, editor_job_id, status, s3_video_url, x_reply_id, created_at, updated_at, tweet_text FROM mention_jobs WHERE status = ?1 ORDER BY updated_at DESC LIMIT ?2"
        } else {
            "SELECT id, tweet_id, user_handle, target_url, editor_job_id, status, s3_video_url, x_reply_id, created_at, updated_at, tweet_text FROM mention_jobs ORDER BY updated_at DESC LIMIT ?1"
        };

        let mut stmt = self.conn.prepare(query)?;
        let rows = if let Some(st) = status {
            stmt.query_map(params![st, limit as i64], |row| {
                Ok(MentionJob {
                    id: row.get(0)?,
                    tweet_id: row.get(1)?,
                    user_handle: row.get(2)?,
                    target_url: row.get(3)?,
                    editor_job_id: row.get(4)?,
                    status: row.get(5)?,
                    s3_video_url: row.get(6)?,
                    x_reply_id: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    tweet_text: row.get(10)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?
        } else {
            stmt.query_map(params![limit as i64], |row| {
                Ok(MentionJob {
                    id: row.get(0)?,
                    tweet_id: row.get(1)?,
                    user_handle: row.get(2)?,
                    target_url: row.get(3)?,
                    editor_job_id: row.get(4)?,
                    status: row.get(5)?,
                    s3_video_url: row.get(6)?,
                    x_reply_id: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    tweet_text: row.get(10)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?
        };

        Ok(rows)
    }

    pub fn count_video_jobs_by_user(&self, user_handle: &str) -> SqlResult<usize> {
        let clean_handle = if user_handle.starts_with('@') {
            user_handle.to_string()
        } else {
            format!("@{}", user_handle)
        };
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM mention_jobs \
             WHERE LOWER(user_handle) = LOWER(?1) \
               AND status != 'conversation' \
               AND status != 'cancelled' \
               AND status != 'failed' \
               AND target_url != 'N/A' \
               AND target_url != ''"
        )?;
        let count: usize = stmt.query_row([clean_handle], |row| row.get(0))?;
        Ok(count)
    }

    pub fn upsert_prospect(&self, p: &Prospect) -> SqlResult<()> {
        self.conn.execute(
            "
            INSERT INTO prospects (handle, name, url, segment, score, stage, last_touch, next_action_date, touches, product_url, last_variant, outcome, notes, why, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)
            ON CONFLICT(handle) DO UPDATE SET
              name = COALESCE(NULLIF(excluded.name, ''), prospects.name),
              url = COALESCE(NULLIF(excluded.url, ''), prospects.url),
              segment = COALESCE(NULLIF(excluded.segment, ''), prospects.segment),
              score = COALESCE(excluded.score, prospects.score),
              stage = COALESCE(NULLIF(excluded.stage, ''), prospects.stage),
              last_touch = COALESCE(excluded.last_touch, prospects.last_touch),
              next_action_date = COALESCE(excluded.next_action_date, prospects.next_action_date),
              touches = COALESCE(excluded.touches, prospects.touches),
              product_url = COALESCE(NULLIF(excluded.product_url, ''), prospects.product_url),
              last_variant = COALESCE(NULLIF(excluded.last_variant, ''), prospects.last_variant),
              outcome = COALESCE(NULLIF(excluded.outcome, ''), prospects.outcome),
              notes = COALESCE(NULLIF(excluded.notes, ''), prospects.notes),
              why = COALESCE(NULLIF(excluded.why, ''), prospects.why),
              updated_at = CURRENT_TIMESTAMP
            ",
            params![
                p.handle,
                p.name.as_deref().unwrap_or(""),
                p.url.as_deref().unwrap_or(""),
                p.segment.as_deref().unwrap_or("founder"),
                p.score.unwrap_or(0),
                p.stage.as_deref().unwrap_or("new"),
                p.last_touch.as_deref(),
                p.next_action_date.as_deref(),
                p.touches.unwrap_or(0),
                p.product_url.as_deref().unwrap_or(""),
                p.last_variant.as_deref().unwrap_or(""),
                p.outcome.as_deref().unwrap_or(""),
                p.notes.as_deref().unwrap_or(""),
                p.why.as_deref().unwrap_or(""),
            ],
        )?;

        self.mirror_prospect_to_jsonl(p);
        Ok(())
    }

    pub fn get_prospect_by_handle(&self, handle: &str) -> SqlResult<Option<Prospect>> {
        let mut stmt = self.conn.prepare("SELECT id, handle, name, url, segment, score, stage, last_touch, next_action_date, touches, product_url, last_variant, outcome, notes, why, updated_at FROM prospects WHERE handle = ?1")?;
        let mut rows = stmt.query([handle])?;

        if let Some(row) = rows.next()? {
            Ok(Some(Prospect {
                id: row.get(0)?,
                handle: row.get(1)?,
                name: row.get(2)?,
                url: row.get(3)?,
                segment: row.get(4)?,
                score: row.get(5)?,
                stage: row.get(6)?,
                last_touch: row.get(7)?,
                next_action_date: row.get(8)?,
                touches: row.get(9)?,
                product_url: row.get(10)?,
                last_variant: row.get(11)?,
                outcome: row.get(12)?,
                notes: row.get(13)?,
                why: row.get(14)?,
                updated_at: row.get(15)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_prospect_stage(&self, id_or_handle: &str, new_stage: &str) -> SqlResult<bool> {
        if let Ok(num_id) = id_or_handle.parse::<i64>() {
            let count = self.conn.execute(
                "UPDATE prospects SET stage = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![new_stage, num_id],
            )?;
            Ok(count > 0)
        } else {
            let count = self.conn.execute(
                "UPDATE prospects SET stage = ?1, updated_at = CURRENT_TIMESTAMP WHERE LOWER(handle) = LOWER(?2)",
                params![new_stage, id_or_handle],
            )?;
            Ok(count > 0)
        }
    }

    pub fn get_all_prospects(&self, stage: Option<&str>) -> SqlResult<Vec<Prospect>> {
        let query = if stage.is_some() {
            "SELECT id, handle, name, url, segment, score, stage, last_touch, next_action_date, touches, product_url, last_variant, outcome, notes, why, updated_at FROM prospects WHERE stage = ?1 ORDER BY updated_at DESC"
        } else {
            "SELECT id, handle, name, url, segment, score, stage, last_touch, next_action_date, touches, product_url, last_variant, outcome, notes, why, updated_at FROM prospects ORDER BY updated_at DESC"
        };

        let mut stmt = self.conn.prepare(query)?;
        let rows = if let Some(st) = stage {
            stmt.query_map([st], |row| {
                Ok(Prospect {
                    id: row.get(0)?,
                    handle: row.get(1)?,
                    name: row.get(2)?,
                    url: row.get(3)?,
                    segment: row.get(4)?,
                    score: row.get(5)?,
                    stage: row.get(6)?,
                    last_touch: row.get(7)?,
                    next_action_date: row.get(8)?,
                    touches: row.get(9)?,
                    product_url: row.get(10)?,
                    last_variant: row.get(11)?,
                    outcome: row.get(12)?,
                    notes: row.get(13)?,
                    why: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(Prospect {
                    id: row.get(0)?,
                    handle: row.get(1)?,
                    name: row.get(2)?,
                    url: row.get(3)?,
                    segment: row.get(4)?,
                    score: row.get(5)?,
                    stage: row.get(6)?,
                    last_touch: row.get(7)?,
                    next_action_date: row.get(8)?,
                    touches: row.get(9)?,
                    product_url: row.get(10)?,
                    last_variant: row.get(11)?,
                    outcome: row.get(12)?,
                    notes: row.get(13)?,
                    why: row.get(14)?,
                    updated_at: row.get(15)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?
        };

        Ok(rows)
    }

    pub fn log_activity(&self, act: &Activity) -> SqlResult<()> {
        let ts = act
            .ts
            .clone()
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
        self.conn.execute(
            "
            INSERT INTO activities (ts, action, handle, segment, variant, detail, result)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                ts,
                act.action,
                act.handle.as_deref().unwrap_or(""),
                act.segment.as_deref().unwrap_or(""),
                act.variant.as_deref().unwrap_or(""),
                act.detail.as_deref().unwrap_or(""),
                act.result.as_deref().unwrap_or("ok"),
            ],
        )?;

        self.mirror_activity_to_jsonl(&Activity {
            ts: Some(ts),
            action: act.action.clone(),
            handle: act.handle.clone(),
            segment: act.segment.clone(),
            variant: act.variant.clone(),
            detail: act.detail.clone(),
            result: act.result.clone(),
            id: None,
        });

        Ok(())
    }

    pub fn get_activities(&self, limit: usize) -> SqlResult<Vec<Activity>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, ts, action, handle, segment, variant, detail, result FROM activities ORDER BY id DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map([limit], |row| {
            Ok(Activity {
                id: row.get(0)?,
                ts: row.get(1)?,
                action: row.get(2)?,
                handle: row.get(3)?,
                segment: row.get(4)?,
                variant: row.get(5)?,
                detail: row.get(6)?,
                result: row.get(7)?,
            })
        })?
        .collect::<SqlResult<Vec<_>>>()?;

        Ok(rows)
    }

    pub fn delete_item(&self, item_type: &str, id: &str) -> SqlResult<bool> {
        let table = match item_type {
            "prospect" => "prospects",
            "mention_job" => "mention_jobs",
            "activity" => "activities",
            _ => return Ok(false),
        };
        if let Ok(num_id) = id.parse::<i64>() {
            let query = format!("DELETE FROM {} WHERE id = ?1", table);
            self.conn.execute(&query, [num_id])?;
        } else {
            let col = if item_type == "prospect" { "handle" } else { "tweet_id" };
            let query = format!("DELETE FROM {} WHERE {} = ?1", table, col);
            self.conn.execute(&query, [id])?;
        }
        Ok(true)
    }

    pub fn get_insights(&self) -> SqlResult<String> {
        let mut stmt = self.conn.prepare("SELECT content FROM insights WHERE id = 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Ok(String::new())
        }
    }

    pub fn upsert_insights(&self, content: &str) -> SqlResult<()> {
        self.conn.execute(
            "
            INSERT INTO insights (id, content, updated_at) VALUES (1, ?1, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET content = excluded.content, updated_at = CURRENT_TIMESTAMP
            ",
            [content],
        )?;
        Ok(())
    }

    fn mirror_activity_to_jsonl(&self, act: &Activity) {
        let state_dir = self
            .repo_root
            .join(".opencode")
            .join("skills")
            .join("x-growth")
            .join("state");
        let _ = fs::create_dir_all(&state_dir);
        let log_file = state_dir.join("activity-log.jsonl");

        if let Ok(json) = serde_json::to_string(act) {
            use std::io::Write;
            if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log_file) {
                let _ = writeln!(f, "{}", json);
            }
        }
    }

    fn mirror_prospect_to_jsonl(&self, prospect: &Prospect) {
        let state_dir = self
            .repo_root
            .join(".opencode")
            .join("skills")
            .join("x-growth")
            .join("state");
        let _ = fs::create_dir_all(&state_dir);
        let jsonl_file = state_dir.join("prospects.jsonl");

        let mut prospects = Vec::new();
        if jsonl_file.exists() {
            if let Ok(content) = fs::read_to_string(&jsonl_file) {
                for line in content.lines() {
                    if let Ok(p) = serde_json::from_str::<Prospect>(line) {
                        prospects.push(p);
                    }
                }
            }
        }

        if let Some(idx) = prospects
            .iter()
            .position(|p| p.handle.eq_ignore_ascii_case(&prospect.handle))
        {
            prospects[idx] = prospect.clone();
        } else {
            prospects.push(prospect.clone());
        }

        use std::io::Write;
        if let Ok(mut f) = fs::File::create(jsonl_file) {
            for p in prospects {
                if let Ok(json) = serde_json::to_string(&p) {
                    let _ = writeln!(f, "{}", json);
                }
            }
        }
    }
}
