mod config;
mod db;
mod discover;
mod safety;
mod server;
mod x_api;

use clap::{Parser, Subcommand};
use db::{Activity, Database, MentionJob, Prospect};

#[derive(Parser)]
#[command(name = "pitch-cli")]
#[command(about = "Pure Rust CLI & Webhook Server for Pitch X/Twitter Agent Automation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Axum webhook dispatcher server (default port 8790, or PORT env var)
    Server {
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// SQLite Database Memory Operations
    Db {
        #[command(subcommand)]
        cmd: DbCommands,
    },

    /// Search X for ICP SaaS prospects and save to SQLite CRM
    Discover {
        #[arg(long)]
        dry: bool,
        #[arg(short, long, default_value = "5")]
        max: usize,
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

        Commands::Discover { dry, max } => {
            let _ = discover::discover_prospects(max, dry).await;
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

        let delivered = mention_jobs
            .iter()
            .filter(|j| j.status == "delivered")
            .count();
        let rendering = mention_jobs
            .iter()
            .filter(|j| j.status == "rendering")
            .count();

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
