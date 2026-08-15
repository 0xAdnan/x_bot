mod config;
mod db;
mod discover;
mod inbox;
mod pitch_mcp;
mod safety;
mod server;
mod worker;
mod x_api;

use clap::{Parser, Subcommand};
use db::{Activity, Database, MentionJob, Prospect};
use x_api::XApiClient;

#[derive(Parser)]
#[command(name = "pitch-cli")]
#[command(about = "Pure Rust CLI & Webhook Server for Pitch X/Twitter Agent Automation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Axum Webhook Server on port 8080 (or PORT env var)
    Server {
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// X API v2 Operations (auto OAuth2 token refresh)
    XApi {
        #[command(subcommand)]
        cmd: XApiCommands,
    },

    /// Pitch MCP API Operations
    Mcp {
        #[command(subcommand)]
        cmd: McpCommands,
    },

    /// SQLite Database Memory Operations
    Db {
        #[command(subcommand)]
        cmd: DbCommands,
    },

    /// Trigger one-shot mention inbox ingestion
    Inbox {
        #[arg(long)]
        dry: bool,
        #[arg(long)]
        no_ack: bool,
    },

    /// Trigger one-shot demo video rendering delivery worker
    Worker {
        #[arg(long)]
        dry: bool,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Search X for ICP SaaS prospects and save to SQLite CRM
    Discover {
        #[arg(long)]
        dry: bool,
        #[arg(short, long, default_value = "5")]
        max: usize,
    },

    /// Unified trigger pass (Inbox + Worker + Stats)
    Trigger {
        #[arg(long)]
        dry: bool,
        #[arg(long)]
        no_ack: bool,
    },

    /// Check remaining daily action budget & rolling burst caps
    Budget,

    /// Check, trip, or reset account safety circuit breaker
    CircuitBreaker {
        #[arg(long)]
        trip: Option<String>,
        #[arg(long)]
        reset: bool,
    },

    /// Print database summary & stats
    Sync,
}

#[derive(Subcommand)]
enum XApiCommands {
    /// Get authenticated user details
    Me,
    /// Force an OAuth2 token refresh
    Refresh,
    /// Lookup user by username
    Lookup { username: String },
    /// Fetch recent mentions
    Mentions {
        #[arg(long)]
        since_id: Option<String>,
        #[arg(short, long, default_value = "20")]
        max: usize,
    },
    /// Reply to a tweet
    Reply {
        tweet_id: String,
        #[arg(short, long)]
        text: String,
        #[arg(long)]
        dry: bool,
    },
    /// Post a tweet
    Post {
        #[arg(short, long)]
        text: String,
        #[arg(long)]
        dry: bool,
    },
    /// Like a tweet
    Like {
        tweet_id: String,
        #[arg(long)]
        dry: bool,
    },
    /// Search recent tweets
    Search {
        query: String,
        #[arg(short, long, default_value = "10")]
        max: usize,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    /// Trigger AI video generation for a URL
    Create {
        url: String,
        #[arg(short, long)]
        instructions: Option<String>,
        #[arg(short, long)]
        voice: Option<String>,
        #[arg(short, long)]
        background: Option<String>,
        #[arg(long)]
        browser_header: Option<String>,
        #[arg(short, long)]
        theme: Option<String>,
    },
    /// Create an AI Launch Video project
    CreateLaunch {
        name: String,
        #[arg(short, long)]
        prompt: String,
    },
    /// Query launch video project status
    LaunchStatus { name: String },
    /// Query job rendering status
    Status { job_id: String },
    /// Check account credit balance
    Credits,
}

#[derive(Subcommand)]
enum DbCommands {
    /// List mention jobs
    Jobs {
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Get a mention job by tweet ID
    GetJob { tweet_id: String },
    /// Upsert a mention job JSON
    UpsertJob { json: String },
    /// List CRM prospects
    Prospects {
        #[arg(short, long)]
        stage: Option<String>,
    },
    /// Get prospect details by handle
    GetProspect { handle: String },
    /// Upsert prospect JSON
    UpsertProspect { json: String },
    /// Log an activity JSON
    Log { json: String },
    /// Get or set adaptive memory insights
    Insights {
        #[command(subcommand)]
        cmd: Option<InsightsCommands>,
    },
}

#[derive(Subcommand)]
enum InsightsCommands {
    Get,
    Set { content: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => {
            server::run_server(port).await;
        }

        Commands::XApi { cmd } => {
            let mut x_client = XApiClient::new();
            match cmd {
                XApiCommands::Me => match x_client.get_me().await {
                    Ok(me) => println!("{}", serde_json::to_string_pretty(&me).unwrap()),
                    Err(e) => eprintln!("[X API Error]: {}", e),
                },
                XApiCommands::Refresh => match x_client.refresh_token().await {
                    Ok(_) => println!("[OK] Token refreshed successfully"),
                    Err(e) => eprintln!("[X API Error]: {}", e),
                },
                XApiCommands::Lookup { username } => match x_client.lookup_user(&username).await {
                    Ok(u) => println!("{}", serde_json::to_string_pretty(&u).unwrap()),
                    Err(e) => eprintln!("[X API Error]: {}", e),
                },
                XApiCommands::Mentions { since_id, max } => {
                    match x_client.get_mentions(since_id.as_deref(), max).await {
                        Ok(m) => println!("{}", serde_json::to_string_pretty(&m).unwrap()),
                        Err(e) => eprintln!("[X API Error]: {}", e),
                    }
                }
                XApiCommands::Reply { tweet_id, text, dry } => {
                    if dry {
                        println!("[DRY RUN] Reply to {}: {}", tweet_id, text);
                    } else {
                        match x_client.post_tweet(&text, Some(&tweet_id)).await {
                            Ok(id) => println!("[OK] Posted reply ID: {}", id),
                            Err(e) => eprintln!("[X API Error]: {}", e),
                        }
                    }
                }
                XApiCommands::Post { text, dry } => {
                    if dry {
                        println!("[DRY RUN] Post: {}", text);
                    } else {
                        match x_client.post_tweet(&text, None).await {
                            Ok(id) => println!("[OK] Published tweet ID: {}", id),
                            Err(e) => eprintln!("[X API Error]: {}", e),
                        }
                    }
                }
                XApiCommands::Like { tweet_id, dry } => {
                    if dry {
                        println!("[DRY RUN] Like {}", tweet_id);
                    } else {
                        match x_client.like_tweet(&tweet_id).await {
                            Ok(true) => println!("[OK] Liked tweet {}", tweet_id),
                            Ok(false) => println!("[WARN] Could not confirm like for {}", tweet_id),
                            Err(e) => eprintln!("[X API Error]: {}", e),
                        }
                    }
                }
                XApiCommands::Search { query, max } => {
                    match x_client.search_recent(&query, max).await {
                        Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                        Err(e) => eprintln!("[X API Error]: {}", e),
                    }
                }
            }
        }

        Commands::Mcp { cmd } => match cmd {
            McpCommands::Create { url, instructions, voice, background, browser_header, theme } => {
                match pitch_mcp::create_demo_video(
                    &url,
                    instructions.as_deref(),
                    voice.as_deref(),
                    background.as_deref(),
                    browser_header.as_deref(),
                    theme.as_deref(),
                ).await {
                    Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                    Err(e) => eprintln!("[Pitch MCP Error]: {}", e),
                }
            }
            McpCommands::CreateLaunch { name, prompt } => {
                match pitch_mcp::create_launch_video(&name, &prompt, None).await {
                    Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                    Err(e) => eprintln!("[Pitch MCP Error]: {}", e),
                }
            }
            McpCommands::LaunchStatus { name } => {
                match pitch_mcp::get_launch_video_status(&name).await {
                    Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                    Err(e) => eprintln!("[Pitch MCP Error]: {}", e),
                }
            }
            McpCommands::Status { job_id } => match pitch_mcp::get_job_status(&job_id).await {
                Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                Err(e) => eprintln!("[Pitch MCP Error]: {}", e),
            },
            McpCommands::Credits => match pitch_mcp::get_credits().await {
                Ok(res) => println!("{}", serde_json::to_string_pretty(&res).unwrap()),
                Err(e) => eprintln!("[Pitch MCP Error]: {}", e),
            },
        },

        Commands::Db { cmd } => match Database::open() {
            Ok(db) => match cmd {
                DbCommands::Jobs { status } => {
                    let jobs = db
                        .get_mention_jobs_by_status(status.as_deref(), 100)
                        .unwrap_or_default();
                    println!("{}", serde_json::to_string_pretty(&jobs).unwrap());
                }
                DbCommands::GetJob { tweet_id } => {
                    let job = db
                        .get_mention_job_by_tweet_id(&tweet_id)
                        .unwrap_or_default();
                    println!("{}", serde_json::to_string_pretty(&job).unwrap());
                }
                DbCommands::UpsertJob { json } => match serde_json::from_str::<MentionJob>(&json) {
                    Ok(job) => {
                        let _ = db.upsert_mention_job(&job);
                        println!("[OK] Job saved to SQLite DB");
                    }
                    Err(e) => eprintln!("[DB JSON Error]: {}", e),
                },
                DbCommands::Prospects { stage } => {
                    let prospects = db.get_all_prospects(stage.as_deref()).unwrap_or_default();
                    println!("{}", serde_json::to_string_pretty(&prospects).unwrap());
                }
                DbCommands::GetProspect { handle } => {
                    let prospect = db.get_prospect_by_handle(&handle).unwrap_or_default();
                    println!("{}", serde_json::to_string_pretty(&prospect).unwrap());
                }
                DbCommands::UpsertProspect { json } => {
                    match serde_json::from_str::<Prospect>(&json) {
                        Ok(p) => {
                            let _ = db.upsert_prospect(&p);
                            println!("[OK] Prospect saved to SQLite DB");
                        }
                        Err(e) => eprintln!("[DB JSON Error]: {}", e),
                    }
                }
                DbCommands::Log { json } => match serde_json::from_str::<Activity>(&json) {
                    Ok(act) => {
                        let _ = db.log_activity(&act);
                        println!("[OK] Activity logged to SQLite DB");
                    }
                    Err(e) => eprintln!("[DB JSON Error]: {}", e),
                },
                DbCommands::Insights { cmd } => match cmd {
                    Some(InsightsCommands::Set { content }) => {
                        let _ = db.upsert_insights(&content);
                        println!("[OK] Insights updated");
                    }
                    _ => {
                        let content = db.get_insights().unwrap_or_default();
                        println!("{}", content);
                    }
                },
            },
            Err(e) => eprintln!("[DB Error]: {}", e),
        },

        Commands::Inbox { dry, no_ack } => {
            if let Err(e) = inbox::process_mention_inbox(dry, no_ack).await {
                eprintln!("[Inbox Error]: {}", e);
            }
        }

        Commands::Worker { dry, limit } => {
            if let Err(e) = worker::advance_rendering_queue(dry, limit).await {
                eprintln!("[Worker Error]: {}", e);
            }
        }

        Commands::Discover { dry, max } => {
            if let Err(e) = discover::discover_prospects(max, dry).await {
                eprintln!("[Discover Error]: {}", e);
            }
        }

        Commands::Trigger { dry, no_ack } => {
            println!("⚡ [EXECUTING UNIFIED RUST TRIGGER PASS]");
            if let Err(e) = inbox::process_mention_inbox(dry, no_ack).await {
                eprintln!("[Inbox Error]: {}", e);
            }
            if let Err(e) = worker::advance_rendering_queue(dry, 10).await {
                eprintln!("[Worker Error]: {}", e);
            }
            print_sync_summary();
        }

        Commands::Budget => {
            let b = safety::budget_check();
            println!("{}", serde_json::to_string_pretty(&b).unwrap());
        }

        Commands::CircuitBreaker { trip, reset } => {
            if reset {
                safety::circuit_breaker_reset();
            } else if let Some(reason) = trip {
                safety::circuit_breaker_trip(&reason);
            } else {
                let (paused, msg) = safety::circuit_breaker_status();
                println!("{}", msg);
                if paused {
                    std::process::exit(1);
                }
            }
        }

        Commands::Sync => {
            print_sync_summary();
        }
    }
}

fn print_sync_summary() {
    if let Ok(db) = Database::open() {
        let mention_jobs = db.get_mention_jobs_by_status(None, 100).unwrap_or_default();
        let prospects = db.get_all_prospects(None).unwrap_or_default();

        let delivered = mention_jobs.iter().filter(|j| j.status == "delivered").count();
        let rendering = mention_jobs.iter().filter(|j| j.status == "rendering").count();

        println!("\n=== [PITCH X BOT SQLITE DATABASE SUMMARY] ===");
        println!(
            "Mention Jobs Total : {} (Delivered: {}, Rendering: {})",
            mention_jobs.len(),
            delivered,
            rendering
        );
        println!("CRM Prospects Total: {}", prospects.len());
    }
}
