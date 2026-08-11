use crate::{config::Config, db::Database};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{net::SocketAddr, process::Stdio, sync::atomic::{AtomicUsize, Ordering}, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct AppState {
    pub client_secret: String,
}

#[derive(Deserialize)]
pub struct CrcQuery {
    pub crc_token: Option<String>,
}

#[derive(Serialize)]
pub struct CrcResponse {
    pub response_token: String,
}

#[derive(Deserialize)]
pub struct TriggerPayload {
    pub action: Option<String>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub server: String,
    pub message: String,
    pub timestamp: String,
}

pub async fn run_server(port_override: Option<u16>) {
    tracing_subscriber::fmt::init();

    let cfg = Config::load();
    let port = port_override.unwrap_or_else(|| {
        std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8790)
    });

    let client_secret = if !cfg.x_client_secret.is_empty() {
        cfg.x_client_secret.clone()
    } else {
        "default_secret".to_string()
    };

    let state = Arc::new(AppState { client_secret });

    let webhook_routes = Router::new()
        .route("/x", get(handle_crc).post(handle_x_webhook))
        .route("/pitch", post(handle_pitch_webhook))
        .route("/trigger", post(handle_manual_trigger))
        .route("/health", get(handle_health))
        .route("/stats", get(handle_stats))
        .with_state(state);

    let app = Router::new()
        .nest("/api/webhook", webhook_routes)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Pitch webhook dispatcher (spawns opencode sessions) listening on http://{}", addr);
    info!("📌 Unified webhook base: /api/webhook (x, pitch, trigger, health, stats)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_crc(
    Query(query): Query<CrcQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let crc_token = match query.crc_token {
        Some(token) if !token.is_empty() => token,
        _ => {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "status": "active",
                    "endpoint": "/api/webhook/x",
                    "info": "X Webhook CRC Endpoint Active"
                })),
            )
                .into_response();
        }
    };

    info!("Received X Webhook CRC token check: {}", crc_token);

    let mut mac = match HmacSha256::new_from_slice(state.client_secret.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            error!("HMAC initialization failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "HMAC error"})),
            )
                .into_response();
        }
    };

    mac.update(crc_token.as_bytes());
    let result = mac.finalize();
    let b64_digest = STANDARD.encode(result.into_bytes());
    let response_token = format!("sha256={}", b64_digest);

    info!("Generated CRC Response Token: {}", response_token);

    (StatusCode::OK, Json(CrcResponse { response_token })).into_response()
}

static ACTIVE_SESSIONS: AtomicUsize = AtomicUsize::new(0);

fn max_sessions() -> usize {
    std::env::var("MAX_OPENCODE_SESSIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
}

fn spawn_opencode_session(task: &str) -> Result<(), String> {
    if ACTIVE_SESSIONS.load(Ordering::Relaxed) >= max_sessions() {
        return Err(format!("at session cap ({})", max_sessions()));
    }

    let cfg = Config::load();
    let sessions_dir = cfg.repo_root.join("data").join("sessions");
    if std::fs::create_dir_all(&sessions_dir).is_err() {
        return Err("could not create data/sessions".to_string());
    }

    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f");
    let log_path = sessions_dir.join(format!("{}.log", ts));
    let log = std::fs::File::create(&log_path).map_err(|e| format!("log file: {}", e))?;

    let prompt = format!(
        "You are handling an event dispatched by the PITCH webhook server. Load the x-mention skill \
         and execute its flow end-to-end for this event. Task:\n\n{}",
        task
    );

    info!("[Dispatcher] spawning opencode session (log: {})", log_path.display());
    ACTIVE_SESSIONS.fetch_add(1, Ordering::Relaxed);

    let pid_path = log_path.clone();
    let task_owned = task.to_string();
    std::thread::spawn(move || {
        match std::process::Command::new("opencode")
            .arg("run")
            .arg(&prompt)
            .current_dir(&cfg.repo_root)
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone().unwrap_or_else(|_| {
                std::fs::File::create(&pid_path).unwrap_or_else(|_| {
                    std::fs::OpenOptions::new().append(true).open("/dev/null").unwrap()
                })
            })))
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(mut child) => {
                let _ = child.wait();
                info!("[Dispatcher] opencode session finished: {}", task_owned);
            }
            Err(e) => {
                error!("[Dispatcher] failed to spawn opencode session: {}", e);
            }
        }
        ACTIVE_SESSIONS.fetch_sub(1, Ordering::Relaxed);
    });

    Ok(())
}

async fn handle_x_webhook(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("[Dispatcher] Received X Webhook POST (mention event)");

    let mentions_summary = body["tweet_create_events"]
        .as_array()
        .map(|events| events.len())
        .unwrap_or(0);
    let detail = extract_mention_info(&body);

    let task = match detail {
        Some(m) => format!(
            "A mention was received from @{} (tweet id {}) with text:\n{}\n\n\
             Follow the x-mention skill: post an instant ack reply, create the demo video via the \
             pitch MCP server for the product URL in the tweet, and deliver the rendered S3 video \
             link as an X reply. Only generate a video if the tweet contains a product URL.",
            m.handle, m.tweet_id, m.text
        ),
        None => format!(
            "An X webhook fired with {} tweet event(s) but no actionable mention could be parsed. \
             Check recent mentions via the xmcp server and handle any new @trypitchdotco mentions \
             per the x-mention skill.",
            mentions_summary
        ),
    };

    tokio::spawn(async move {
        if let Err(e) = spawn_opencode_session(&task) {
            error!("[Dispatcher] {} — task skipped: {}", e, task);
        }
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": "Mention event received; opencode session dispatched",
            "tweet_events": mentions_summary,
            "dispatched": true
        })),
    )
}

async fn handle_pitch_webhook(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let job_id = body["jobId"]
        .as_str()
        .or_else(|| body["id"].as_str())
        .unwrap_or("unknown");
    let status = body["status"].as_str().unwrap_or("unknown");
    info!("[Dispatcher] Pitch MCP completion callback: job {} status {}", job_id, status);

    let task = format!(
        "Pitch MCP reported a rendering event for job {} with status {}. Follow the x-mention skill: \
         run `awb status` first, look up the matching mention job, and if the render completed \
         deliver the S3 video link as an X reply via agent-webbridge (Testing profile).",
        job_id, status
    );

    tokio::spawn(async move {
        if let Err(e) = spawn_opencode_session(&task) {
            error!("[Dispatcher] {} — task skipped: {}", e, task);
        }
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": "Pitch completion callback received; opencode session dispatched",
            "job_id": job_id,
            "dispatched": true
        })),
    )
}

async fn handle_manual_trigger(payload: Option<Json<TriggerPayload>>) -> impl IntoResponse {
    let action = payload
        .and_then(|p| p.action.clone())
        .unwrap_or_else(|| "mentions".to_string());

    info!("[Dispatcher] Manual trigger requested with action: {}", action);

    let task = match action.as_str() {
        "session" | "growth" => {
            "Run the x-growth session loop: boot guardrails, check budget and circuit breaker, \
             survey state, then run the session loop (prospect / engage / outreach / content / \
             community) as appropriate."
                .to_string()
        }
        "discover" => "Run the x-prospect skill to search for new ICP SaaS prospects and log them."
            .to_string(),
        _ => {
            "Check recent X mentions via the xmcp server and handle any new @trypitchdotco mentions \
             per the x-mention skill."
                .to_string()
        }
    };

    tokio::spawn(async move {
        if let Err(e) = spawn_opencode_session(&task) {
            error!("[Dispatcher] {} — task skipped: {}", e, task);
        }
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "action": action,
            "message": "Trigger dispatched to an opencode session",
            "dispatched": true
        })),
    )
}

struct MentionInfo {
    handle: String,
    tweet_id: String,
    text: String,
}

fn extract_mention_info(body: &serde_json::Value) -> Option<MentionInfo> {
    let events = body.get("tweet_create_events")?.as_array()?;
    let mine = body.get("user")?.get("id").and_then(|i| i.as_str());

    for ev in events {
        let author_id = ev.get("user")?.get("id").and_then(|i| i.as_str());
        let handle = ev
            .get("user")?
            .get("screen_name")
            .and_then(|s| s.as_str())?;
        let tweet_id = ev.get("id").and_then(|i| i.as_str())?;
        let text = ev.get("text").and_then(|t| t.as_str())?;

        if handle.eq_ignore_ascii_case("trypitchdotco") {
            continue;
        }
        if let Some(author) = author_id {
            if let Some(m) = mine {
                if author == m {
                    continue;
                }
            }
        }

        let text_lower = text.to_lowercase();
        if !text_lower.contains("@trypitchdotco") {
            continue;
        }

        return Some(MentionInfo {
            handle: handle.to_string(),
            tweet_id: tweet_id.to_string(),
            text: text.to_string(),
        });
    }
    None
}

async fn handle_health() -> impl IntoResponse {
    let ts = chrono::Utc::now().to_rfc3339();
    (
        StatusCode::OK,
        Json(StatusResponse {
            status: "ok".to_string(),
            server: "pitch-webhook-server (rust / axum)".to_string(),
            message: "Pitch Webhook Server active and operational".to_string(),
            timestamp: ts,
        }),
    )
}

async fn handle_stats() -> impl IntoResponse {
    if let Ok(db) = Database::open() {
        let mention_jobs = db.get_mention_jobs_by_status(None, 100).unwrap_or_default();
        let prospects = db.get_all_prospects(None).unwrap_or_default();

        let delivered = mention_jobs.iter().filter(|j| j.status == "delivered").count();
        let rendering = mention_jobs.iter().filter(|j| j.status == "rendering").count();

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "mention_jobs_total": mention_jobs.len(),
                "delivered": delivered,
                "rendering": rendering,
                "prospects_total": prospects.len()
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "status": "error", "message": "Failed to open SQLite database" })),
        )
    }
}
