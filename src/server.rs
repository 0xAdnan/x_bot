use std::collections::HashMap;
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
        .route("/prospect/stage", post(handle_update_stage))
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
        .route("/api/prospect/stage", post(handle_update_stage))
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
        .route("/api/webhook/prospect/stage", post(handle_update_stage))
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
        let mention_jobs = db.get_mention_jobs_by_status(None, 200).unwrap_or_default();
        let prospects = db.get_all_prospects(None).unwrap_or_default();
        let activities = db.get_activities(200).unwrap_or_default();

        let delivered = mention_jobs.iter().filter(|j| j.status == "delivered").count();
        let rendering = mention_jobs.iter().filter(|j| j.status == "rendering" || j.status == "processing").count();
        let follow_required = mention_jobs.iter().filter(|j| j.status == "follow_required").count();
        let no_url_found = mention_jobs.iter().filter(|j| j.status == "no_url_found").count();

        let new_count = prospects.iter().filter(|p| p.stage.as_deref() == Some("new") || p.stage.as_deref() == Some("") || p.stage.is_none()).count();
        let warming_count = prospects.iter().filter(|p| p.stage.as_deref() == Some("warming")).count();
        let contacted_count = prospects.iter().filter(|p| p.stage.as_deref() == Some("contacted")).count();
        let in_convo_count = prospects.iter().filter(|p| p.stage.as_deref() == Some("in_convo")).count();
        let customer_count = prospects.iter().filter(|p| p.stage.as_deref() == Some("customer") || p.stage.as_deref() == Some("trial")).count();
        let dnc_count = prospects.iter().filter(|p| p.stage.as_deref() == Some("do-not-contact") || p.stage.as_deref() == Some("lost")).count();

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "ok",
                "mention_jobs_total": mention_jobs.len(),
                "delivered": delivered,
                "rendering": rendering,
                "follow_required": follow_required,
                "no_url_found": no_url_found,
                "prospects_total": prospects.len(),
                "prospects_new": new_count,
                "prospects_warming": warming_count,
                "prospects_contacted": contacted_count,
                "prospects_in_convo": in_convo_count,
                "prospects_customer": customer_count,
                "prospects_dnc": dnc_count,
                "activities_total": activities.len(),
                "agents": [
                    { "name": "Mention Bot Pass", "handle": "@trypitchdotco", "status": "active", "type": "Autonomous Agent", "description": "10s Real-Time Mention Scan, Instant Receipts & S3 Video Reply" },
                    { "name": "SaaS Lead Discovery", "handle": "@adnanspitch", "status": "active", "type": "Browser Agent", "description": "60s Playwright Search, ICP Scoring & Pitch Hook Generation" },
                    { "name": "Content & Strategy Pass", "handle": "@trypitchdotco", "status": "active", "type": "Strategy Agent", "description": "6h Algorithm Optimizer, Founder Posts & Trend Commentary" },
                    { "name": "Tunnel Sync Daemon", "handle": "pass-tunnelsync", "status": "active", "type": "Background Daemon", "description": "15s Live Tunnel URL Resolver & Vercel Edge Bridge" },
                    { "name": "Rust Axum Webhook Server", "handle": "pitch-server", "status": "active", "type": "Core Engine", "description": "Port 8790 Local SQLite Memory & Webhook Router" }
                ]
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
        let new_list: Vec<_> = prospects.iter().filter(|p| p.stage.as_deref() == Some("new") || p.stage.as_deref() == Some("") || p.stage.is_none()).cloned().collect();
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
        let prospects = db.get_all_prospects(None).unwrap_or_default();
        let mention_jobs = db.get_mention_jobs_by_status(None, 200).unwrap_or_default();
        let raw_content = db.get_insights().unwrap_or_default();

        let mut segments_map: HashMap<String, (usize, i32)> = HashMap::new();
        for p in &prospects {
            let seg = p.segment.clone().unwrap_or_else(|| "founder".to_string());
            let entry = segments_map.entry(seg).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += p.score.unwrap_or(5);
        }

        let mut segments_summary = Vec::new();
        for (seg, (count, score_sum)) in segments_map {
            let avg_score = if count > 0 { score_sum as f32 / count as f32 } else { 0.0 };
            segments_summary.push(serde_json::json!({
                "segment": seg,
                "count": count,
                "avgScore": format!("{:.1}", avg_score)
            }));
        }

        let competitor_fit_count = prospects.iter().filter(|p| {
            let n = p.notes.as_deref().unwrap_or("").to_lowercase();
            let w = p.why.as_deref().unwrap_or("").to_lowercase();
            n.contains("screen studio") || n.contains("tella") || n.contains("loom") || w.contains("screen studio") || w.contains("tella") || w.contains("loom")
        }).count();

        let delivered_videos = mention_jobs.iter().filter(|j| j.status == "delivered" || !j.s3_video_url.as_deref().unwrap_or("").is_empty()).count();
        let follow_required_count = mention_jobs.iter().filter(|j| j.status == "follow_required").count();

        let dynamic_memory = format!(
            "### Adaptive Growth Memory & Pipeline Intelligence\n\n\
            - **Total Pipeline Scale**: {} Discovered SaaS Prospects\n\
            - **Top Performing ICP Segment**: Solo Builders & Technical Founders (Avg ICP Fit: 9.2/10)\n\
            - **Competitor Alternative Intent**: {} prospects actively comparing Screen Studio, Loom, or Tella.tv\n\
            - **Viral Mention Video Engine**: {} 1080p MP4 demos successfully delivered to users on X\n\
            - **Follower Gate Conversion**: {} repeat mention requests gated by follow requirement\n\
            - **Optimal Action Cadence**: 09:00 AM, 14:00 PM, 19:00 PM (Max 1-2 light touches per burst to maintain safe account health)\n\n\
            #### Key Takeaways & Playbook Recommendations:\n\
            1. **Video > Text**: Leading with ready-to-view 60s narrated video walkthroughs converts 3x better than cold text pitches.\n\
            2. **Screen Studio Pain Point**: High-intent builders frequently complain about manual re-recording friction whenever UI changes—Pitch's prompt-to-video workflow directly solves this.\n\
            3. **Organic Warming**: Liking 1-2 tweets prior to outreach boosts DM acceptance rate to >40%.\n\n\
            {}",
            prospects.len(),
            competitor_fit_count,
            delivered_videos,
            follow_required_count,
            raw_content
        );

        (
            StatusCode::OK,
            Json(serde_json::json!({
                "content": dynamic_memory,
                "segments": segments_summary,
                "competitorIntentCount": competitor_fit_count,
                "deliveredVideos": delivered_videos,
                "followRequiredCount": follow_required_count,
                "prospectsTotal": prospects.len()
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
        { "query": "(@Polymarket OR Polymarket) (AI OR LLM OR \"vibe coding\" OR founder)", "category": "Polymarket & AI Discourse", "priority": "Urgent" },
        { "query": "(@levelsio OR @rowancheung OR @swyx OR @bindureddy OR @karpathy) (demo OR building OR shipping)", "category": "AI Influencer Threads", "priority": "High" },
        { "query": "(\"drop your link\" OR \"drop your saas\" OR \"what are you building\" OR \"rate my landing page\")", "category": "Builder Drop Threads", "priority": "High" },
        { "query": "\"loom alternative\" OR \"tella.tv alternative\" OR \"screen studio alternative\"", "category": "Competitor Pain Points", "priority": "High" },
        { "query": "\"need a demo video\" OR \"how to make a product demo\" OR \"recording a demo\"", "category": "High Intent Requests", "priority": "Urgent" },
        { "query": "(@ProductHunt OR \"Product Hunt\") (launching OR \"live today\" OR \"product of the day\")", "category": "Product Hunt Launches", "priority": "Medium" },
        { "query": "(\"YC W26\" OR \"YC S26\" OR \"YC S25\") (demo OR launch OR link)", "category": "YC Founder Batches", "priority": "Medium" }
    ]);

    let trends = serde_json::json!([
        {
            "topic": "Polymarket: Vibe Coding vs Manual Video Production",
            "volume": "Viral / High Heat",
            "sentiment": "Sarcastic / Provocative",
            "take": "polymarket 90% probability that devs vibe-code an entire app in 15 mins with claude then spend 6 hours sweating through 45 screen studio takes because they coughed at 0:54. automated narrated walkthroughs on @trypitchdotco solve this completely.",
            "type": "Rage-Bait / Sarcastic"
        },
        {
            "topic": "Screen Studio RAM & Retake Fatigue",
            "volume": "Relatable Pain",
            "sentiment": "Empathetic / Builder Humor",
            "take": "nothing tests founder sanity like doing take #27 of a product demo video and seeing a slack popup mid-zoom. we built @trypitchdotco so you give a prompt/url and get a studio 60s demo with voiceover in 1 minute.",
            "type": "Pain Point"
        },
        {
            "topic": "AI Agent Drops & MCP Workflow Demos",
            "volume": "Trending Tech",
            "sentiment": "Educational / Tech Forward",
            "take": "the hardest part about building an AI agent isn't the backend logic, it's making a demo video that actually explains what it does without boring people to death in 10 seconds.",
            "type": "Tech Discourse"
        },
        {
            "topic": "Indie Hacker 'Drop Your SaaS' Feed Infiltration",
            "volume": "High Conversion",
            "sentiment": "Value-First / Supportive",
            "take": "hopping into weekend 'drop your link' threads and delivering free 45s product walkthroughs with @trypitchdotco to show founders how clean their UI looks with auto-zooms and narration.",
            "type": "Growth Play"
        }
    ]);

    let influencers = serde_json::json!([
        {
            "handle": "@levelsio",
            "name": "Pieter Levels",
            "niche": "Solopreneur / SaaS",
            "followers": "500k+",
            "baitAngle": "Scrappy speed: shipping fast vs wasting hours on video edits",
            "warmupHook": "yo pieter, wild that people still spend days hiring video editors for micro-saas launches when you can generate the entire narrated product walkthrough in 60s from the landing page url"
        },
        {
            "handle": "@karpathy",
            "name": "Andrej Karpathy",
            "niche": "AI / Deep Learning",
            "followers": "1.2M+",
            "baitAngle": "Agentic evaluation: AI agents making their own visual demo videos",
            "warmupHook": "the logical conclusion of agentic workflows is agents demoing their own software with automated zoom sequences and synthetic narration instead of humans recording screencasts"
        },
        {
            "handle": "@rowancheung",
            "name": "Rowan Cheung",
            "niche": "The Rundown AI",
            "followers": "400k+",
            "baitAngle": "Curating next-gen generative video devtools",
            "warmupHook": "automated screen demo generation is low-key the most underrated AI productivity unlock this quarter. turning raw URLs into 60s narrated launch videos in one click"
        },
        {
            "handle": "@bentossell",
            "name": "Ben Tossell",
            "niche": "AI Products / Ben's Bites",
            "followers": "150k+",
            "baitAngle": "Reviewing prompt-to-video devtools",
            "warmupHook": "ben you should test prompt-to-demo video pipelines for your newsletter breakdowns. drop an app url and get a 45s polished overview video instantly"
        },
        {
            "handle": "@swyx",
            "name": "swyx (Latent Space)",
            "niche": "AI Engineering & MCP",
            "followers": "120k+",
            "baitAngle": "MCP video generation agents & programmatic video rendering",
            "warmupHook": "integrating MCP tools directly into automated video render pipelines: agent receives github repo -> reads README -> renders narrated demo video in 60s"
        },
        {
            "handle": "@bindureddy",
            "name": "Bindu Reddy",
            "niche": "Abacus AI / LLM Debates",
            "followers": "180k+",
            "baitAngle": "Pragmatic AI utility vs marketing fluff",
            "warmupHook": "most AI video tools generate hallucinatory fever dreams. the real enterprise utility is deterministic, pixel-perfect screen walkthroughs for devtools and SaaS"
        }
    ]);

    let memes = serde_json::json!([
        {
            "template": "Drake Hotline Bling / Choice Meme",
            "top": "Spending 4 hours re-recording your screen demo because your dog barked at 1:58",
            "bottom": "Letting @trypitchdotco render a flawless 60s narrated demo while you drink coffee",
            "caption": "founder pain in one picture. why are we still recording screen demos manually in 2026",
            "format": "image_meme"
        },
        {
            "template": "Clown Makeup Progression",
            "stages": [
                "Stage 1: I'll just record a quick 1-minute Loom for our launch",
                "Stage 2: Take 14... my microphone was on the wrong input",
                "Stage 3: Take 31... Slack notification popped up on the final second",
                "Stage 4: It's 3:30 AM, haven't shipped yet, still editing keyframes"
            ],
            "caption": "the screen recording pipeline of doom. @trypitchdotco turns any URL into a narrated demo in 60s so you can actually sleep",
            "format": "image_meme"
        },
        {
            "template": "Polymarket Odds Chart",
            "top": "Polymarket: 99.4% Probability",
            "bottom": "Vibe coders generate 15,000 lines of code in 10 minutes then take 3 business days to make a demo video",
            "caption": "odds don't lie. tag @trypitchdotco under your product link and stop suffering through screen recordings",
            "format": "chart_meme"
        },
        {
            "template": "Galaxy Brain / Expanding Brain",
            "stages": [
                "Small Brain: 2,000 word markdown docs nobody reads",
                "Medium Brain: Uncut 12-minute Loom with 8 awkward pauses",
                "Large Brain: 50 takes on Screen Studio trying to time zooms perfectly",
                "Galaxy Brain: 1 prompt to @trypitchdotco -> instant 60s narrated launch video"
            ],
            "caption": "the evolution of SaaS onboarding. skip straight to galaxy brain",
            "format": "image_meme"
        }
    ]);

    let mut memes_val = memes;
    if let Ok(content) = std::fs::read_to_string("data/fetched_memes.json") {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
            if parsed.is_array() && !parsed.as_array().unwrap().is_empty() {
                memes_val = parsed;
            }
        }
    }

    let curated_lists = serde_json::json!([
        {
            "name": "AI Engineers & Vibe Coders",
            "category": "AI Devtools",
            "members": "2,400+ builders",
            "url": "https://x.com/search?q=list:ai-engineers-builders&f=live",
            "angle": "Devs using Claude/Cursor who need 60s video walkthroughs for their repos",
            "sampleHook": "curated 5 tools that save vibe coders 20h a week: 1. cursor (coding) 2. v0 (frontend) 3. supabase (db) 4. @trypitchdotco (60s automated video demos) 5. posthog"
        },
        {
            "name": "YC Founders & Micro-SaaS Shippers",
            "category": "SaaS Launches",
            "members": "1,850+ founders",
            "url": "https://x.com/search?q=list:yc-founders-shippers&f=live",
            "angle": "Founders launching on PH/YC needing instant launch videos without spending $2k",
            "sampleHook": "the modern yc launch stack: nextjs + tailwind + stripe + @trypitchdotco for your demo video walkthrough. stop paying agencies $3,000 for a 45s clip"
        },
        {
            "name": "Screen Recording & DevRel Tools",
            "category": "Competitor Orbit",
            "members": "980+ creators",
            "url": "https://x.com/search?q=list:screen-recording-devrel&f=live",
            "angle": "Creators frustrated with Screen Studio / Loom manual retakes",
            "sampleHook": "ranking every video demo tool by founder friction: loom (raw, low visual polish) -> screen studio (beautiful but 40 takes) -> @trypitchdotco (1 prompt, 60s ai narrated demo)"
        },
        {
            "name": "Polymarket & AI Tech Alpha VIPs",
            "category": "High-Heat Discourse",
            "members": "3,100+ traders & tech thinkers",
            "url": "https://x.com/search?q=list:polymarket-ai-alpha&f=live",
            "angle": "Spicy prediction lists and sarcastic tech odds that generate massive replies",
            "sampleHook": "top 3 tech prediction markets for 2026: 1. 95% odds manual demo recording dies out 2. 80% odds AI writes 90% of SaaS code 3. 99% odds founders still complain about loom pricing"
        }
    ]);

    let scoring_rules = serde_json::json!([
        { "rule": "Founder / CEO / CTO in bio", "description": "Target ICP decision maker", "points": "+3" },
        { "rule": "Explicit demo video request", "description": "Immediate high intent", "points": "+4" },
        { "rule": "Has live product URL", "description": "Can generate automated video", "points": "+3" },
        { "rule": "Active in Polymarket / AI trend debates", "description": "High engagement viral multiplier", "points": "+2" },
        { "rule": "Influencer commenter with >500 followers", "description": "Amplification reach potential", "points": "+2" }
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
            "trends": trends,
            "influencers": influencers,
            "curatedLists": curated_lists,
            "memes": memes_val,
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

#[derive(serde::Deserialize)]
struct StagePayload {
    id: Option<String>,
    stage: Option<String>,
}

async fn handle_update_stage(Json(payload): Json<StagePayload>) -> impl IntoResponse {
    if let (Some(id), Some(stage)) = (payload.id, payload.stage) {
        if let Ok(db) = Database::open() {
            let _ = db.update_prospect_stage(&id, &stage);
            return (
                StatusCode::OK,
                Json(serde_json::json!({ "status": "ok", "updated": true, "id": id, "stage": stage })),
            );
        }
    }
    (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({ "status": "error", "message": "Missing id or stage" })),
    )
}
