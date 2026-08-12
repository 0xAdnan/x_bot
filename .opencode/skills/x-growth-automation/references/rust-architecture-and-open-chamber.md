# Pure Rust Architecture & Open Chamber Integration

This reference documents the pure Rust single-binary architecture (`pitch-cli`) on the `rewrite/clean-slate` branch, SQLite database role, embedded Axum webhook server, and Open Chamber scheduler integration.

## 1. Single Compiled Binary (`pitch-cli`)
Instead of separate Python/Node.js/Vercel scripts, the system compiles to a single, high-performance Rust binary (`pitch-cli` built with `tokio`, `axum`, `rusqlite`, `reqwest`, `serde`, `clap`):

```bash
cargo build --release
./target/release/pitch-cli <COMMAND>
```

### Commands
- `pitch-cli server --port 8790`: Runs the embedded Axum HTTP webhook server on port `8790`.
- `pitch-cli trigger`: Executes a unified one-shot pass (Inbox Mention Ingestion + Pitch MCP Video Delivery Worker + Sync Summary).
- `pitch-cli x-api`: Official X API v2 operations (`me`, `refresh`, `lookup`, `mentions`, `reply`, `post`, `like`, `search`) with auto OAuth 2.0 token refresh (`from_path_override`).
- `pitch-cli mcp`: Pitch MCP video rendering operations (`create`, `status`, `credits`).
- `pitch-cli db`: Local SQLite database queries (`jobs`, `prospects`, `log`, `insights`).
- `pitch-cli discover --max 5`: Searches X for ICP SaaS prospects and saves to SQLite CRM.
- `pitch-cli budget` & `circuit-breaker`: Enforces daily action caps and account health checks.

## 2. Embedded Axum Webhook Server (`src/server.rs`)
Listens on `http://0.0.0.0:8790` under route prefix `/api/webhook/`:

| Method | Endpoint | Purpose |
| --- | --- | --- |
| **`GET`** | `/api/webhook/x` | **X CRC Challenge Check** (Account Activity API registration HMAC-SHA256) |
| **`POST`** | `/api/webhook/x` | **X Real-Time Mention Callback** |
| **`POST`** | `/api/webhook/trigger` | **Manual / External Trigger Pass** (`{"action":"mentions"}` or `{"action":"growth"}`) |
| **`GET`** | `/api/webhook/health` | **Server Health & Uptime Check** |
| **`GET`** | `/api/webhook/stats` | **SQLite Database Summary Metrics** |

## 3. Local SQLite Memory (`data/pitch_bot.db`)
Serves as the single source of truth for pipeline state and CRM:
- **`mention_jobs` Table:** Tracks tweet IDs, handles, product URLs, Pitch MCP job IDs, status (`pending -> rendering -> delivered | failed | no_url_found`), and S3 links. Unique constraint on `tweet_id` guarantees idempotency.
- **`prospects` Table:** Lead CRM pipeline (`new -> warming -> contacted -> in_convo -> customer`).
- **`activities` Table:** Logs every action for daily action cap and burst accounting.
- **`insights` Table:** Stores adaptive outreach memory.

## 4. Local Verification & Git Push Policy
- **Local Testing First:** All code edits, Rust builds, and API tests MUST be compiled and verified locally (`cargo check --release`, `/tmp/hermes-verify-*` scripts) before finalizing.
- **No Unprompted Push:** Do NOT execute `git push` to GitHub automatically after committing. Git pushes should occur ONLY when explicitly requested by the user.
