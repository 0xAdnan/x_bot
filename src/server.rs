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
use opencode_rs::types::event::Event;
use opencode_rs::types::message::{Part, PromptPart, PromptRequest};
use opencode_rs::types::session::CreateSessionRequest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
    time::Duration,
};
use tokio::io::AsyncWriteExt;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};

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

    tokio::spawn(check_opencode_server_health());

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
    info!("🚀 Pitch webhook dispatcher (opencode_rs SDK → opencode serve) listening on http://{}", addr);
    info!("📌 Unified webhook base: /api/webhook (x, pitch, trigger, health, stats)");
    info!("📡 Dispatching to opencode server at {}", opencode_base_url());

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

fn opencode_base_url() -> String {
    std::env::var("OPENCODE_URL").unwrap_or_else(|_| "http://127.0.0.1:4096".to_string())
}

async fn dispatch_opencode_session(task: &str) -> Result<(), String> {
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
    let log = tokio::fs::File::create(&log_path)
        .await
        .map_err(|e| format!("log file: {}", e))?;

    let prompt = format!(
        "You are handling an event dispatched by the PITCH webhook server. Load the x-mention skill \
         and execute its flow end-to-end for this event. Task:\n\n{}",
        task
    );

    let client = opencode_rs::ClientBuilder::new()
        .base_url(opencode_base_url())
        .directory(cfg.repo_root.to_string_lossy().to_string())
        .build()
        .map_err(|e| format!("opencode client build: {}", e))?;

    info!("[Dispatcher] dispatching opencode session via SDK (log: {})", log_path.display());
    ACTIVE_SESSIONS.fetch_add(1, Ordering::Relaxed);

    let session = match client
        .sessions()
        .create(&CreateSessionRequest {
            title: Some(format!("pitch-webhook-{}", ts)),
            ..CreateSessionRequest::default()
        })
        .await
    {
        Ok(s) => s,
        Err(e) => {
            ACTIVE_SESSIONS.fetch_sub(1, Ordering::Relaxed);
            return Err(format!("session create: {}", e));
        }
    };

    let mut sub = match client.subscribe_session(&session.id) {
        Ok(s) => s,
        Err(e) => {
            let _ = client.sessions().delete(&session.id).await;
            ACTIVE_SESSIONS.fetch_sub(1, Ordering::Relaxed);
            return Err(format!("session subscribe: {}", e));
        }
    };

    if let Err(e) = client
        .messages()
        .prompt_async(
            &session.id,
            &PromptRequest {
                parts: vec![PromptPart::Text {
                    text: prompt.clone(),
                    synthetic: None,
                    ignored: None,
                    metadata: None,
                }],
                message_id: None,
                model: None,
                agent: None,
                no_reply: None,
                system: None,
                variant: None,
            },
        )
        .await
    {
        let _ = client.sessions().delete(&session.id).await;
        ACTIVE_SESSIONS.fetch_sub(1, Ordering::Relaxed);
        return Err(format!("prompt send: {}", e));
    }

    let task_owned = task.to_string();
    let session_id = session.id.clone();
    tokio::spawn(async move {
        let mut log = log;
        let mut finished = false;
        let _ = log.write_all(format!("[prompt]\n{}\n", prompt).as_bytes()).await;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2 * 60 * 60);

        loop {
            let recv = tokio::time::timeout(Duration::from_secs(30 * 60), sub.recv()).await;
            match recv {
                Ok(Some(ev)) => {
                    if let Some(line) = event_log_line(&ev) {
                        let _ = log.write_all(line.as_bytes()).await;
                    }
                    match &ev {
                        Event::SessionIdle { properties }
                            if properties.session_id == session_id =>
                        {
                            finished = true;
                            break;
                        }
                        Event::SessionError { .. } => break,
                        _ => {}
                    }
                    if tokio::time::Instant::now() > deadline {
                        break;
                    }
                }
                Ok(None) => break,
                Err(_) => break,
            }
        }

        let _ = client.sessions().delete(&session.id).await;
        ACTIVE_SESSIONS.fetch_sub(1, Ordering::Relaxed);
        info!(
            "[Dispatcher] opencode session {} finished (idle={}): {}",
            session.id, finished, task_owned
        );
    });

    Ok(())
}

fn event_log_line(ev: &Event) -> Option<String> {
    match ev {
        Event::MessagePartDelta { properties } => properties.delta.clone(),
        Event::MessagePartUpdated { properties } => match &properties.part {
            Some(Part::Text { text, .. }) => Some(text.clone()),
            _ => None,
        },
        Event::SessionIdle { .. } => Some("[session.idle]".to_string()),
        Event::SessionError { .. } => Some("[session.error]".to_string()),
        _ => None,
    }
}

async fn check_opencode_server_health() {
    let base_url = opencode_base_url();
    let client = opencode_rs::ClientBuilder::new()
        .base_url(base_url.clone())
        .build()
        .ok();
    let Some(client) = client else {
        warn!("[Dispatcher] opencode client failed to build for {}", base_url);
        return;
    };
    match client.global().health().await {
        Ok(h) if h.healthy => {
            info!(
                "[Dispatcher] opencode server healthy at {} (v{})",
                base_url,
                h.version.as_deref().unwrap_or("unknown")
            );
        }
        Ok(_) | Err(_) => {
            warn!(
                "[Dispatcher] opencode server unreachable at {} — start `opencode serve --port 4096`. Incoming webhook events will be rejected with a dispatch error.",
                base_url
            );
        }
    }
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
        if let Err(e) = dispatch_opencode_session(&task).await {
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
        if let Err(e) = dispatch_opencode_session(&task).await {
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
        if let Err(e) = dispatch_opencode_session(&task).await {
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
