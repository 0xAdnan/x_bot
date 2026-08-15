use crate::{
    config::Config,
    db::Database,
    inbox::process_mention_inbox,
    worker::advance_rendering_queue,
};
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
use std::{net::SocketAddr, sync::Arc};
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

    let app = Router::new()
        .route("/x", get(handle_crc).post(handle_x_webhook))
        .route("/x-webhook", get(handle_crc).post(handle_x_webhook))
        .route("/trigger", post(handle_manual_trigger))
        .route("/health", get(handle_health))
        .route("/stats", get(handle_stats))
        .route("/crm", get(handle_crm))
        .route("/mentions", get(handle_mentions))
        .route("/insights", get(handle_insights))
        .route("/activity", get(handle_activity))
        .route("/research", get(handle_research))
        .route("/delete", post(handle_delete))
        .route("/api/x", get(handle_crc).post(handle_x_webhook))
        .route("/api/x-webhook", get(handle_crc).post(handle_x_webhook))
        .route("/api/trigger", post(handle_manual_trigger))
        .route("/api/health", get(handle_health))
        .route("/api/stats", get(handle_stats))
        .route("/api/crm", get(handle_crm))
        .route("/api/mentions", get(handle_mentions))
        .route("/api/insights", get(handle_insights))
        .route("/api/activity", get(handle_activity))
        .route("/api/research", get(handle_research))
        .route("/api/delete", post(handle_delete))
        .route("/api/webhook/x", get(handle_crc).post(handle_x_webhook))
        .route("/api/webhook/x-webhook", get(handle_crc).post(handle_x_webhook))
        .route("/api/webhook/trigger", post(handle_manual_trigger))
        .route("/api/webhook/health", get(handle_health))
        .route("/api/webhook/stats", get(handle_stats))
        .route("/api/webhook/crm", get(handle_crm))
        .route("/api/webhook/mentions", get(handle_mentions))
        .route("/api/webhook/insights", get(handle_insights))
        .route("/api/webhook/activity", get(handle_activity))
        .route("/api/webhook/research", get(handle_research))
        .route("/api/webhook/delete", post(handle_delete))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Pitch Rust Webhook Server listening on http://{}", addr);
    info!("📌 Routes available under /api/webhook/ (x, trigger, health, stats)");

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

async fn handle_x_webhook(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    info!("Received X Webhook POST Payload: {:?}", body);

    tokio::spawn(async move {
        let _ = process_mention_inbox(false, false).await;
        let _ = advance_rendering_queue(false, 10).await;
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": "X Webhook mention event received and trigger pass scheduled",
            "triggered": true
        })),
    )
}

async fn handle_manual_trigger(payload: Option<Json<TriggerPayload>>) -> impl IntoResponse {
    let action = payload
        .and_then(|p| p.action.clone())
        .unwrap_or_else(|| "mentions".to_string());

    info!("Manual webhook trigger requested with action: {}", action);

    let action_clone = action.clone();
    tokio::spawn(async move {
        if action_clone == "session" || action_clone == "growth" {
            let _ = crate::discover::discover_prospects(5, false).await;
        }
        let _ = process_mention_inbox(false, false).await;
        let _ = advance_rendering_queue(false, 10).await;
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "action": action,
            "message": "Trigger pass dispatched in background",
            "triggered": true
        })),
    )
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

async fn handle_crm() -> impl IntoResponse {
    if let Ok(db) = Database::open() {
        let prospects = db.get_all_prospects(None).unwrap_or_default();
        let new_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("new")).cloned().collect();
        let warming_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("warming")).cloned().collect();
        let contacted_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("contacted")).cloned().collect();
        let in_convo_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("in_convo")).cloned().collect();
        let trial_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("trial")).cloned().collect();
        let customer_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("customer")).cloned().collect();
        let dnc_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("do-not-contact")).cloned().collect();
        let lost_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("lost")).cloned().collect();

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "prospects": prospects,
                "total": prospects.len(),
                "stages": {
                    "new": new_list,
                    "warming": warming_list,
                    "contacted": contacted_list,
                    "in_convo": in_convo_list,
                    "trial": trial_list,
                    "customer": customer_list,
                    "do-not-contact": dnc_list,
                    "lost": lost_list
                }
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Database error" })),
        )
    }
}

async fn handle_mentions() -> impl IntoResponse {
    if let Ok(db) = Database::open() {
        let jobs = db.get_mention_jobs_by_status(None, 100).unwrap_or_default();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "jobs": jobs,
                "total": jobs.len()
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Database error" })),
        )
    }
}

async fn handle_insights() -> impl IntoResponse {
    if let Ok(db) = Database::open() {
        let content = db.get_insights().unwrap_or_default();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "content": content
            })),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Database error" })),
        )
    }
}

async fn handle_activity() -> impl IntoResponse {
    if let Ok(db) = Database::open() {
        let acts = db.get_activities(100).unwrap_or_default();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "activities": acts,
                "total": acts.len()
            })),
        )
    } else {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "activities": [], "total": 0 })),
        )
    }
}

async fn handle_research() -> impl IntoResponse {
    let queries = serde_json::json!([
        { "query": "\"loom alternative\" OR \"tella.tv alternative\"", "category": "Competitor Mentions", "priority": "High" },
        { "query": "\"need a demo video\" OR \"how to make a product demo\"", "category": "High Intent", "priority": "High" },
        { "query": "YC W26 OR \"launching on product hunt\"", "category": "SaaS Launches", "priority": "Medium" }
    ]);
    let scoring_rules = serde_json::json!([
        { "rule": "Founder / CEO / CTO in bio", "description": "Target ICP decision maker", "points": "+3" },
        { "rule": "Explicit demo video request", "description": "Immediate high intent", "points": "+4" },
        { "rule": "Has live product URL", "description": "Can generate automated video", "points": "+3" }
    ]);

    let mut queue = Vec::new();
    if let Ok(db) = Database::open() {
        let prospects = db.get_all_prospects(Some("new")).unwrap_or_default();
        for p in prospects.iter().take(10) {
            queue.push(serde_json::json!({
                "account": "@trypitchdotco",
                "status": "ready",
                "type": "Demo Outreach Hook",
                "hook": p.why.clone().unwrap_or_else(|| format!("Created custom pitch hook for {}", p.handle)),
                "preset": "Charon | Light | Ocean"
            }));
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "queries": queries,
            "scoringRules": scoring_rules,
            "contentQueue": queue
        })),
    )
}

#[derive(serde::Deserialize)]
struct DeletePayload {
    #[serde(rename = "type")]
    item_type: Option<String>,
    id: Option<String>,
}

async fn handle_delete(Json(payload): Json<DeletePayload>) -> impl IntoResponse {
    if let (Some(item_type), Some(id)) = (payload.item_type, payload.id) {
        if let Ok(db) = Database::open() {
            let _ = db.delete_item(&item_type, &id);
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "status": "ok", "deleted": true })),
    )
}
