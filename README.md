# PITCH — OpenCode X/Twitter Growth & AI Video Demo Bot

A pure, high-performance **Rust CLI toolset, Webhook Server, and OpenCode Agent** for **PITCH** (https://trypitch.co) — the AI video editor that turns written walkthroughs into studio-quality narrated demo MP4s.

---

## 🌟 System Overview

This system operates natively inside OpenCode using:
1. **100% Pure Rust Architecture:** Single compiled binary (`./target/release/pitch-cli`) containing X API v2 client, Pitch MCP client, safety engine, and Axum Webhook Server.
2. **Local SQLite Database Memory:** Stored at `data/pitch_bot.db`. No Supabase or external database servers required.
3. **Decoupled Workflow (Zero Context Window Bloat):**
   * **Part 1 (Real-Time Webhook-Driven):** Mention ingestion, receipt replies, and video generation handled automatically via the embedded Rust Webhook Server. Zero cron needed!
   * **Part 2 (Focused Scheduled Passes):** Low-context OpenCode agent sessions for ICP prospect discovery, warm-up outreach, and founder commentary.

---

## 🏗️ Architecture

```
                                 ┌─────────────────────────────┐
                                 │     X Webhook / Trigger     │
                                 └──────────────┬──────────────┘
                                                │
                          POST /webhookbase/x-webhook OR /trigger
                                                │
                                 ┌──────────────▼──────────────┐
                                 │     Rust Webhook Server     │
                                 │  (pitch-cli server - Axum)  │
                                 └──────────────┬──────────────┘
                                                │
                            Dispatches Execution / Agent Session
                                                │
                                 ┌──────────────▼──────────────┐
                                 │   OpenCode Agent (x-growth) │
                                 │    pitch-cli CLI Toolset    │
                                 └──────────────┬──────────────┘
                                                │
                                 ┌──────────────▼──────────────┐
                                 │     SQLite DB Memory        │
                                 │    (data/pitch_bot.db)      │
                                 └─────────────────────────────┘
```

---

## 🚀 Quick Start

### 1. Requirements
* Rust 1.80+ (`cargo` / `rustc`)
* OpenCode CLI

### 2. Setup Configuration
Copy `.env.example` to `.env` and fill in your credentials:
```bash
cp .env.example .env
```

```env
X_USER_ACCESS_TOKEN=your_oauth2_user_access_token
X_USER_REFRESH_TOKEN=your_oauth2_user_refresh_token
X_CLIENT_ID=your_x_client_id
X_CLIENT_SECRET=your_x_client_secret
PITCH_API_KEY=pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB
PORT=8790
SQLITE_DB_PATH=./data/pitch_bot.db
```

### 3. Build Binary
```bash
cargo build --release
```

---

## 🛠️ Command Matrix (`pitch-cli`)

Location: `./target/release/pitch-cli` (or run via `cargo run --release -- <command>`)

### 🌐 Rust Webhook Server
Starts the Axum Webhook Server listening on `http://0.0.0.0:8790`:
```bash
./target/release/pitch-cli server
```
* **CRC Challenge:** `GET /webhookbase/x-webhook?crc_token=...`
* **Real-time Mention Callback:** `POST /webhookbase/x-webhook`
* **Manual Trigger:** `POST /webhookbase/trigger` (`{"action": "mentions"}` or `{"action": "session"}`)
* **Health Check:** `GET /webhookbase/health`
* **Pipeline Summary:** `GET /webhookbase/stats`

### 💾 SQLite Database Memory
```bash
./target/release/pitch-cli db jobs                # List all mention jobs
./target/release/pitch-cli db jobs --status rendering  # List active rendering jobs
./target/release/pitch-cli db prospects           # List CRM prospects
./target/release/pitch-cli db insights            # Read adaptive memory insights
```

### 🐦 X API v2 (Auto OAuth2 Token Refresh)
```bash
./target/release/pitch-cli x-api me               # Verify profile
./target/release/pitch-cli x-api mentions         # Fetch recent mentions
./target/release/pitch-cli x-api reply <id> --text "..." # Post reply
./target/release/pitch-cli x-api post --text "..." # Post tweet
./target/release/pitch-cli x-api search "query"   # Search X
```

### 🎬 Pitch MCP API
```bash
./target/release/pitch-cli mcp create <url> "<instructions>"  # Trigger AI video demo
./target/release/pitch-cli mcp status <job_id>               # Poll video render status
./target/release/pitch-cli mcp credits                        # Check credits
```

### 🛡️ Safety & Budget Enforcers
```bash
./target/release/pitch-cli circuit-breaker        # Check circuit breaker status
./target/release/pitch-cli circuit-breaker --reset # Reset circuit breaker
./target/release/pitch-cli budget                 # Check daily action caps & rolling burst limit
```

### ⚡ Unified Pipelines & Triggers
```bash
./target/release/pitch-cli inbox                  # Fetch mentions & trigger demo jobs
./target/release/pitch-cli worker                 # Deliver completed demo videos
./target/release/pitch-cli trigger                # Run unified pass (inbox + worker + stats)
./target/release/pitch-cli discover               # Discover SaaS ICP prospects
./target/release/pitch-cli sync                   # Display database summary
```

---

## 🤖 OpenCode Agent Execution (`x-growth`)

Run the OpenCode agent for focused sessions:

```bash
# 1. Process Mentions & Deliver Videos
opencode run --agent x-growth "process mentions"

# 2. Discover SaaS ICP Prospects
opencode run --agent x-growth "discover prospects"

# 3. Warm-Up & Outbound DMs
opencode run --agent x-growth "engage warm prospects"

# 4. Content & Founder Commentary
opencode run --agent x-growth "publish founder commentary"
```
