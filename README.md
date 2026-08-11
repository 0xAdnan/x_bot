# PITCH X Growth Engine (x_bot)

Developer documentation for the automated X/Twitter growth pipeline behind
[PITCH](https://trypitch.co). Everything runs from one compiled Rust binary
(`pitch-cli`) plus a local SQLite database. This doc covers every request flow,
how the pipeline is scheduled (webhook real-time + cron polling), how webhook
endpoints trigger specific skills, and how those skills accomplish the business
goals of PITCH.

## System overview

```
                        ┌──────────────────────────────────────┐
                        │        pitch-cli  (Rust / Axum)      │
   X Account Activity   │                                      │
   webhook  ───────────►│   server (port 8790)                 │
   (POST /api/webhook/x)│      │                               │
                        │      ▼                               │
   cron / manual ──────►│   trigger ──► inbox ──► workers      │
   (pitch-cli trigger)  │                │  │                  │
                        │                ▼  ▼                  │
                        │   discover (outbound prospecting)   │
                        │                                      │
                        │  x_api (X API v2, OAuth2 refresh)    │
                        │  pitch_mcp (api.trypitch.co/mcp)     │
                        │  safety (budget + circuit breaker)   │
                        └──────────────┬───────────────────────┘
                                       │
                                       ▼
                        data/pitch_bot.db  (SQLite: mention_jobs,
                                            prospects, activities, insights)
```

Two ways work gets triggered:

1. **Real-time webhook** — X calls `POST /api/webhook/x` when someone mentions
   `@trypitchdotco`. The server spawns a background pass (`inbox` then
   `worker`) and returns `200` immediately.
2. **Cron / polling** — the CLI subcommands (`trigger`, `inbox`, `worker`,
   `discover`) are invoked on a schedule by cron (see [Schedules & cron](#schedules--cron)).
   This is the recovery net: if a webhook POST is lost, the next polled pass
   picks up the same mentions (the DB makes every job idempotent).

## Webhook Endpoints → Skill Mapping → Business Accomplishments

The system exposes 5 HTTP webhook routes via Axum. Below is the mapping from
each endpoint to its triggered skill, execution pipeline, and business outcome:

| Endpoint | Triggered Skill | Pipeline Execution | Business Accomplishment (ROI) |
|---|---|---|---|
| **`POST /api/webhook/x`** | `x-mention` | 1. Receipt ack reply tweet<br>2. Pitch MCP render (1080p MP4)<br>3. In-thread S3 video delivery | **Inbound Viral Demo Funnel:** Users receive a studio-quality video demo of their URL in <2 min. Public replies showcase PITCH's magic to all followers, driving viral social proof & signups. |
| **`POST /api/webhook/trigger`**<br>`{"action": "growth"}` | `x-prospect` | 1. Search X API v2 for SaaS launch keywords<br>2. Lead scoring (1–10 fit)<br>3. Upsert to SQLite (`stage: new`) | **High-Intent Lead Pipeline:** Automatically builds a pipeline of SaaS founders launching products who need video walkthroughs. |
| **`POST /api/webhook/trigger`**<br>`{"action": "mentions"}` | `x-mention` | 1. Fetch unhandled mentions<br>2. Submit pending renders to MCP<br>3. Deliver finished video replies | **Inbound Recovery Net:** Guarantees zero missed mentions even if an X webhook callback fails or drops. |
| **Scheduled Agent Pass**<br>`(opencode run --agent x-growth)` | `x-engage`<br>`x-outreach` | 1. Like/reply to warm leads (`stage: warming`)<br>2. Custom DM with user's URL walkthrough<br>3. Update CRM (`stage: contacted / in_convo`) | **Paid Subscriber Conversion:** Nurtures SaaS founders in public, then converts them in DMs into paid PITCH subscribers. |
| **Daily Scheduled Pass**<br>`(opencode run --agent x-growth)` | `x-content`<br>`x-community` | 1. Post product updates & tech takes<br>2. Help indie hackers in public threads<br>3. Drive organic profile impressions | **Brand Authority & Awareness:** Builds trust, follower growth, and organic inbound traffic for `@trypitchdotco`. |
| **`GET /api/webhook/x`** | System Utility | HMAC-SHA256 CRC token verification challenge | **X API Compliance:** Required by X Account Activity API to register webhooks. |
| **`GET /api/webhook/health`** | System Utility | Liveness probe returning HTTP 200 OK | **Deployment Health:** Ensures server is operational. |
| **`GET /api/webhook/stats`** | System Utility | Pipeline metrics query from SQLite | **Observability:** Monitors total jobs, renders, and CRM prospects. |

## Request flows

### Flow A — X mention webhook (`x-mention` skill)

Endpoints: `GET /api/webhook/x` (CRC handshake) and `POST /api/webhook/x`
(event callback).

**CRC registration (GET).** `GET /api/webhook/x?crc_token=<token>` returns
`{"response_token": "sha256=<base64(HMAC-SHA256(crc_token, X_CLIENT_SECRET))>"}`.
Handled by `handle_crc` in `src/server.rs:93`.

**Event callback (POST).** Payload body is ignored; the handler
(`src/server.rs:136`) immediately spawns a background task:

1. `process_mention_inbox(false, false)` (`src/inbox.rs:22`)
2. `advance_rendering_queue(false, 10)` (`src/worker.rs:7`)

Then returns `200 {"status":"ok","triggered":true}` to X.

**Inbox stage** (`process_mention_inbox`):

1. Fetches up to 20 recent mentions via X API v2 (`GET /users/{id}/mentions`).
2. Skips mentions authored by `@trypitchdotco` and tweets already present in
   `mention_jobs` (idempotency — never double-bills a render or double-replies).
3. Extracts a URL from the tweet text (`https?://...` first, then a bare
   domain, `src/inbox.rs:8`).
4. **No URL / `s3.trypitch.co` URL** → records the job with status
   `no_url_found` and stops for that tweet.
5. **Valid URL** → posts an instant receipt reply on X
   (`x-api reply`, honoring `--dry` / `--no-ack`), then calls the Pitch MCP
   tool `create_demo_video` (`src/pitch_mcp.rs:62`).
   - Returns a `jobId` → job saved as `rendering`.
   - No `jobId` → job saved as `submitted` (retried by the next worker pass).

**Worker stage** (`advance_rendering_queue`): polls the Pitch MCP tool
`get_job` for every job in `rendering` or `submitted` status:

- `COMPLETED` → extract S3 artifact URL (`src/pitch_mcp.rs:93`; falls back to
  `https://trypitch.co/editor/<jobId>`), post the demo link back to the
  original tweet via X API, mark the job `delivered` (stores `s3_video_url`
  and `x_reply_id`).
- `FAILED` / `ERROR` → mark the job `failed`.
- Otherwise → leave as-is for the next worker pass.

### Flow B — Manual trigger endpoint

`POST /api/webhook/trigger` with optional JSON body `{"action": "..."}`
(`src/server.rs:154`). Spawns a background pass:

- `action` = `growth` or `session` → runs `discover_prospects(5, false)` first.
- Always then runs `process_mention_inbox(false, false)` and
  `advance_rendering_queue(false, 10)`.

Responds `200 {"status":"ok","action":...,"triggered":true}` and returns
immediately (all heavy work happens in the spawned task). Useful for calling
from an external scheduler or `curl` for an on-demand pass.

### Flow C — Outbound prospecting (`x-prospect` skill)

`discover_prospects(max_per_query, dry_run)` (`src/discover.rs:67`), triggered
via `pitch-cli discover` or the webhook `action=growth` path:

1. Runs the fixed ICP search-query list (`src/discover.rs:7`) against X search
   recent (`GET /tweets/search/recent`).
2. Skips `@trypitchdotco`, `@adnanspitch`, and already-known handles.
3. Scores each lead 1–10 (`calculate_lead_score`, `src/discover.rs:32`) using
   URL presence, competitor keywords (tella, screen studio, loom, ...), and
   intent keywords (need, looking for, how to, launched, building).
4. Builds a pre-cooked DM hook (`generate_pitch_hook`) and upserts the prospect
   into `prospects` (stage `new`, segment `founder`) plus `state/prospects.jsonl`.

### Flow D — Health, stats, checks

- `GET /api/webhook/health` → `{"status":"ok", ...}` (`src/server.rs:181`).
- `GET /api/webhook/stats` → counts of jobs (`mention_jobs_total`,
  `delivered`, `rendering`) and `prospects_total` (`src/server.rs:194`).
- `pitch-cli sync` → same summary to stdout.
- `pitch-cli budget` → remaining daily caps per action + rolling 1-hour burst
  count (`src/safety.rs:157`). Writes are blocked when a cap is 0 or the
  breaker is tripped.
- `pitch-cli circuit-breaker` → `OK to run` / `PAUSED`; trips are recorded in
  `state/circuit-breaker.jsonl` and 3 trips in 24h create `state/HARD_STOP`
  (hard pause) until `--reset`.

## Routes table

Server binds `0.0.0.0:{PORT}`, where `PORT` defaults to `8790`
(`src/server.rs:52`). All routes are mounted under both `/api/webhook/...`
and `/webhook/...` (`src/server.rs:79`).

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/api/webhook/x` | X webhook CRC challenge (HMAC-SHA256 token) |
| `POST` | `/api/webhook/x` | X mention event → spawn inbox + worker pass |
| `POST` | `/api/webhook/trigger` | Manual/on-demand pass (`{"action":"mentions"\|"growth"}`) |
| `GET` | `/api/webhook/health` | Liveness probe |
| `GET` | `/api/webhook/stats` | DB pipeline counts |

## Job state machine

`mention_jobs` table (one row per tweet):

```
no_url_found ─► (terminal; nothing to render)
      submitted ─► rendering ─► delivered
                     │
                     └────────► failed
```

`no_ack` / `--dry` flag affects only the receipt reply, not job creation.
`--dry` never calls X or Pitch MCP, only prints what it would do.

## Database

`data/pitch_bot.db` (override with `SQLITE_DB_PATH`), WAL mode, tables
created at startup (`src/db.rs:67`):

- **`mention_jobs`** — idempotent demo pipeline state (unique `tweet_id`).
- **`prospects`** — CRM; upsert keyed on `handle`, mirrored to
  `.opencode/skills/x-growth/state/prospects.jsonl`.
- **`activities`** — every action (`ts`, `action`, `handle`, `result`), the
  input for `budget`. Mirrored to `activity-log.jsonl`.
- **`insights`** — adaptive memory blob (single-row table, `id=1`).

## Schedules & cron

The binary has **no internal scheduler**. Automation = the always-on webhook
server for real-time flow + cron for the passes listed below. Directory used in
examples: `/opt/pitch-xbot` (your repo root).

### 1. Run the real-time server

```bash
./target/release/pitch-cli server          # binds 0.0.0.0:8790
# or: PORT=9000 ./target/release/pitch-cli server
```

Run under a process manager (launchd on macOS, systemd on Linux; or e.g.
`nohup`, tmux, pm2-anywhere). It must be reachable by X on a public URL
(e.g. ngrok tunnel or reverse proxy in front of port 8790) for webhooks to
arrive.

### 2. Fallback polling cron (recovery net)

Even with the webhook live, schedule a polled pass every 5 minutes. This
catches lost webhook posts, re-polls slow render jobs, and delivers videos
that the webhook-spawned worker didn't have time to finish:

```cron
# every 5 min: ingest mentions + advance render queue + DB summary
*/5 * * * * cd /opt/pitch-xbot && ./target/release/pitch-cli trigger >> /var/log/pitch/trigger.log 2>&1
```

`trigger` = `inbox` + `worker` + `sync` in one pass (`src/main.rs:346`).

If you prefer narrower jobs (`--dry` first to verify):

```cron
*/10 * * * * cd /opt/pitch-xbot && ./target/release/pitch-cli inbox   >> /var/log/pitch/inbox.log 2>&1
*/3  * * * * cd /opt/pitch-xbot && ./target/release/pitch-cli worker  >> /var/log/pitch/worker.log 2>&1
```

### 3. Outbound prospecting

ICP discovery is rate-sensitive — spread it out, never burst. `discover` is
blocked while `budget` shows 0 remaining for the `discover` action (cap 40/day
default, scaled by ramp factor during cold start — see `src/safety.rs:177`).

```cron
# every 3 hours, 08:00-23:00 ASIA/KOLKATA
0 8-23/3 * * * cd /opt/pitch-xbot && ./target/release/pitch-cli discover --max 5 >> /var/log/pitch/discover.log 2>&1
```

### 4. Safety & observability

```cron
# daily heartbeat: verify breaker open + budget healthy + pipeline summary
0 9 * * * cd /opt/pitch-xbot && ./target/release/pitch-cli circuit-breaker && ./target/release/pitch-cli budget && ./target/release/pitch-cli sync >> /var/log/pitch/daily.log 2>&1
```

Manual controls when needed:

```bash
./target/release/pitch-cli circuit-breaker --trip "reason"   # pause automation
./target/release/pitch-cli circuit-breaker --reset           # resume (also clears trip log)
```

Recommended schedule summary:

| Pass | Cadence | Rationale |
|---|---|---|
| `server` | always-on | real-time mention → demo flow |
| `trigger` | every 5 min | recovery net; idempotent; advances renders |
| `discover` | every 3 h | ICP lead discovery, spaced for rate limits |
| `circuit-breaker`+`budget`+`sync` | daily 09:00 | health + budget heartbeat |
| `/api/webhook/trigger` | on manual demand | arbitrary on-demand pass |

Add spacing discipline if you push cron further: X burst hygiene in this
project requires ≥90s between consecutive writes; the safety engine enforces a
10-actions/hour burst signal via `budget`.

## Setup

### 1. Requirements

- Rust 1.80+ (`cargo` / `rustc`)
- A reachable public endpoint for the webhook server (or tunnel)

### 2. Environment (`.env`, gitignored)

Keys read by the code (`src/config.rs`):

```env
X_CLIENT_ID=your_x_client_id
X_CLIENT_SECRET=your_x_client_secret
X_USER_ACCESS_TOKEN=your_oauth2_user_access_token
X_USER_REFRESH_TOKEN=your_oauth2_user_refresh_token
X_USER_ID=your_x_user_id
X_OPERATOR_HANDLE=@trypitchdotco
PITCH_API_KEY=your_pitch_api_key        # required; do NOT rely on the src/config.rs fallback
SQLITE_DB_PATH=./data/pitch_bot.db      # optional override
PORT=8790                               # optional override
```

- `X_USER_ACCESS_TOKEN` / `X_USER_REFRESH_TOKEN`: must be functional. The
  client auto-refreshes on a `401` (`src/x_api.rs:143`) but cannot mint
  tokens from scratch. **Known state (2026-08):** stored OAuth2 user tokens are
  expired/invalid — `pitch-cli x-api me` returns `401`. Fresh tokens must be
  saved to `.env` before any X API v2 write flow (receipt replies, video
  delivery, discovery) works.
- Legacy cache keys (`X_API_KEY`, `X_API_SECRET`, `X_BEARER_TOKEN`,
  `X_WEBHOOK_ID`) are unused by the code.
- Never commit `.env` or expose `PITCH_API_KEY` / OAuth2 tokens in logs.

### 3. Build & verify

```bash
cargo build --release
./target/release/pitch-cli sync                 # empty DB → zeros
./target/release/pitch-cli circuit-breaker      # expect "OK to run"
./target/release/pitch-cli budget               # confirm caps + 0 used
./target/release/pitch-cli mcp credits          # confirm Pitch MCP auth
./target/release/pitch-cli server --port 8790 & # boot server
curl http://localhost:8790/api/webhook/health   # ok
curl "http://localhost:8790/api/webhook/x?crc_token=test"  # sha256=...
```

### 4. Wire the webhook in X

Register `https://<public-url>/api/webhook/x` as the Account Activity webhook
URL in your X app. X will call `GET` (CRC) during registration and `POST` on
every mention thereafter.

## Automation checklist (end to end)

1. `.env` populated with working OAuth2 user tokens + `PITCH_API_KEY`.
2. `cargo build --release` passes.
3. `pitch-cli circuit-breaker` says `OK to run`; `pitch-cli budget` shows
   remaining caps.
4. `pitch-cli server` is always-on behind a public URL.
5. `*/5` cron runs `pitch-cli trigger`; `*/3`h cron runs `pitch-cli discover`.
6. Watch `/var/log/pitch/*.log` + `GET /api/webhook/stats`; trip/reset the
   breaker from cron output on anomalies.
