# PITCH X Growth Engine (x_bot)

Developer documentation for the automated X/Twitter growth pipeline behind
[PITCH](https://trypitch.co). Everything runs from one compiled Rust binary
(`pitch-cli`) plus a local SQLite database, and every event is handed off to a
fresh `opencode run` session that executes the matching skill. This doc covers
every request flow, how the pipeline is scheduled (webhook real-time + cron),
how webhook endpoints dispatch opencode sessions, and how the x-* skills
accomplish the business goals of PITCH.

## System overview

```
                X/X account activity webhook
                (POST /api/webhook/x)  ──────────────┐
                                                    ▼
                    Pitch MCP completion  ──►  pitch-cli server (Rust / Axum,
                    (POST /api/webhook/pitch)         port 8790)
                                                      │  dispatcher: spawns
                                                      ▼
                                          opencode run "<event task>"  (capped by
                                          MAX_OPENCODE_SESSIONS, default 3)
                                                      │
                                                      ▼
                                x-* skill session (x-mention / x-growth / x-prospect)
                                ├─ pitch MCP server (create_demo_video, get_job)
                                ├─ xmcp MCP server (reads)
                                └─ agent-webbridge "Testing" (all X writes)
                                                      │
                                                      ▼
                                      data/pitch_bot.db (SQLite mention_jobs,
                                          prospects, activities, insights)
```

Two ways work gets triggered:

1. **Real-time webhook** — X calls `POST /api/webhook/x` when someone mentions
   `@trypitchdotco`. The server hands the mention to a fresh `opencode run`
   session running the `x-mention` skill and returns `200` immediately.
2. **Manual / cron** — `pitch-cli server` also exposes `/api/webhook/trigger`
   (curl on demand), and the `discover` CLI polls X for ICP leads. There is no
   Rust mention pipeline anymore: `inbox`, `worker`, `trigger` subcommands and
   `src/pitch_mcp.rs` were deleted in favor of webhook-dispatched opencode
   sessions.

## Webhook Endpoints → Skill Mapping → Business Accomplishments

All webhook routes are exposed under the **single unified base `/api/webhook`**
(no duplicated `/webhook/...` mounts or `/x-webhook` aliases) because X
registers one callback URL for an Account Activity subscription. Every incoming
event dispatches a fresh `opencode run` session that executes the mapped skill.
Below is the mapping from each endpoint to its dispatched session, execution
flow, and business outcome:

| Endpoint | Triggered Skill | Pipeline Execution | Business Accomplishment (ROI) |
|---|---|---|---|
| **`POST /api/webhook/x`** | `x-mention` | 1. Dispatch opencode session with mention<br>2. Receipt ack reply via webbridge `Testing`<br>3. Pitch MCP render (1080p MP4)<br>4. In-thread S3 video delivery | **Inbound Viral Demo Funnel:** Users receive a studio-quality video demo of their URL in <2 min. Public replies showcase PITCH's magic to all followers, driving viral social proof & signups. |
| **`POST /api/webhook/pitch`** | `x-mention` | 1. Dispatch opencode session on completion<br>2. Session polls `get_job`, posts S3 delivery reply | **Fast Delivery:** A render completion can prompt an immediate delivery pass instead of waiting for the next poll. |
| **`POST /api/webhook/trigger`**<br>`{"action": "growth"}` | `x-growth` | 1. Dispatch opencode session running the x-growth session loop<br>2. Prospect/search, engage, outreach as appropriate<br>3. Upsert CRM (`state/prospects.jsonl` + SQLite) | **High-Intent Lead Pipeline:** Builds a pipeline of SaaS founders who need video walkthroughs. |
| **`POST /api/webhook/trigger`**<br>`{"action": "mentions"}` | `x-mention` | 1. Dispatch opencode session to check recent mentions<br>2. Handle any unprocessed `@trypitchdotco` mentions | **Inbound Recovery Net:** Recovers mentions even if an X webhook callback fails or drops. |
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
Handled by `handle_crc` in `src/server.rs`.

**Event callback (POST).** The handler (`src/server.rs`) parses
`tweet_create_events` for a mention of `@trypitchdotco` from another account,
builds a task prompt for `spawn_opencode_session`, and returns
`200 {"status":"ok","dispatched":true}` immediately.

**Session dispatch** (`spawn_opencode_session`, `src/server.rs`): spawns
`opencode run "<task>"` in the repo root (so it picks up `opencode.jsonc`,
skills, and the `x-growth` agent), caps concurrency via `MAX_OPENCODE_SESSIONS`
(default 3), and writes the session's stdout to `data/sessions/<timestamp>.log`.

Inside the session, the `x-mention` skill does:

1. `awb status` first; then posts an instant receipt reply via
   `agent-webbridge` (`Testing` profile).
2. Calls the Pitch MCP tool `create_demo_video` (`get_job` / `get_credits` are
   available too) for the product URL in the tweet — no URL or an
   `s3.trypitch.co` URL → records the job as `no_url_found`.
3. Polls `get_job` until `COMPLETED` (5 min initial sleep, then 2-min
   intervals), extracts the S3 artifact URL, and delivers it as an in-thread
   reply via `agent-webbridge`. FAILED → `failed`, otherwise retries.
4. Records each step in SQLite (`mention_jobs`) and the CRM state files.

**Render completion (POST `/api/webhook/pitch`).** If `PITCH_WEBHOOK_URL` is
set on `create_demo_video`, Pitch MCP posts the completion callback here. The
handler dispatches another opencode session to look up the job and deliver the
finished video. Sessions also poll on their own, so this callback is optional.

### Flow B — Manual trigger endpoint

`POST /api/webhook/trigger` with optional JSON body `{"action": "..."}`
(`src/server.rs`). Dispatches an opencode session:

- `action` = `growth` or `session` → runs the `x-growth` session loop.
- `action` = `discover` → runs the `x-prospect` skill.
- otherwise → runs the `x-mention` recovery pass.

Responds `200 {"status":"ok","action":...,"dispatched":true}` immediately.

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

- `GET /api/webhook/health` → `{"status":"ok", ...}` (`src/server.rs:203`).
- `GET /api/webhook/stats` → counts of jobs (`mention_jobs_total`,
  `delivered`, `rendering`) and `prospects_total` (`src/server.rs:216`).
- `pitch-cli sync` → same summary to stdout.
- `pitch-cli budget` → remaining daily caps per action + rolling 1-hour burst
  count (`src/safety.rs:157`). Writes are blocked when a cap is 0 or the
  breaker is tripped.
- `pitch-cli circuit-breaker` → `OK to run` / `PAUSED`; trips are recorded in
  `state/circuit-breaker.jsonl` and 3 trips in 24h create `state/HARD_STOP`
  (hard pause) until `--reset`.

## Routes table

Server binds `0.0.0.0:{PORT}`, where `PORT` defaults to `8790`
(`src/server.rs:52`). All webhook routes live under the single canonical base
`/api/webhook` (`src/server.rs:79`) — no `/webhook/...` mount, no `/x-webhook`
alias. X registers `https://<public-url>/api/webhook/x` as its one callback URL,
and Pitch MCP completion posts to `https://<public-url>/api/webhook/pitch`
(`PITCH_WEBHOOK_URL`).

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/api/webhook/x` | X webhook CRC challenge (HMAC-SHA256 token) |
| `POST` | `/api/webhook/x` | X mention event → dispatch `x-mention` opencode session |
| `POST` | `/api/webhook/pitch` | Pitch MCP render completion → dispatch delivery session |
| `POST` | `/api/webhook/trigger` | Manual/on-demand dispatch (`{"action":"mentions"\|"growth"\|"discover"}`) |
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

Job rows are written by the dispatched opencode sessions as they work; the
`x-mention` skill is responsible for idempotency (never double-reply or
double-bill a render for the same tweet).

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
server, which dispatches an `opencode run` session per event. Directory used in
examples: `/opt/pitch-xbot` (your repo root).

### 1. Run the real-time server

```bash
./target/release/pitch-cli server          # binds 0.0.0.0:8790
# or: PORT=9000 ./target/release/pitch-cli server
```

Run under a process manager (launchd on macOS, systemd on Linux; or e.g.
`nohup`, tmux, pm2-anywhere). It must be reachable by X on a public URL
(e.g. ngrok tunnel or reverse proxy in front of port 8790) for webhooks to
arrive. Each event spawns `opencode run` (concurrency capped by
`MAX_OPENCODE_SESSIONS`, logs under `data/sessions/`), so the host also needs
the opencode CLI and the model provider authed.

### 2. Recovery net

The webhook is the primary path. If an event is dropped, hit the trigger
endpoint on demand — e.g. a cron line:

```cron
# every 5 min: re-check recent mentions + deliver any completed renders
*/5 * * * * curl -s -X POST http://127.0.0.1:8790/api/webhook/trigger -H 'Content-Type: application/json' -d '{"action":"mentions"}' >> /var/log/pitch/trigger.log 2>&1
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
| `server` | always-on | real-time mention → demo via dispatched sessions |
| `/api/webhook/trigger` `{"action":"mentions"}` | every 5 min | recovery net; re-checks mentions + delivers renders |
| `discover` | every 3 h | ICP lead discovery, spaced for rate limits |
| `circuit-breaker`+`budget`+`sync` | daily 09:00 | health + budget heartbeat |

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
X_OPERATOR_HANDLE=@trypitchdotco
X_USERNAME=your_x_username
X_PASSWORD=your_x_password
PITCH_API_KEY=your_pitch_api_key        # used by opencode.jsonc {env:PITCH_API_KEY}
PITCH_WEBHOOK_URL=https://<public-url>/api/webhook/pitch   # optional completion callback for Pitch MCP renders
SQLITE_DB_PATH=./data/pitch_bot.db      # optional override
PORT=8790                               # optional override
MAX_OPENCODE_SESSIONS=3                 # optional: concurrent dispatched sessions
```

- `PITCH_API_KEY` is consumed by the `pitch` MCP server in `opencode.jsonc`
  (`{env:PITCH_API_KEY}`); opencode resolves it from the process env, so export
  it before launching opencode (e.g. `set -a; source .env; set +a`).
- `X_CLIENT_ID` / `X_CLIENT_SECRET`: feed the `xmcp` MCP server's `xurl` bridge
  OAuth2 PKCE login (`opencode.jsonc`). First run opens the browser to sign in;
  tokens are cached in `~/.xurl` and auto-refreshed. `xmcp` is used for reads;
  all X **writes** go through `agent-webbridge` (`Testing` profile).
- `PITCH_WEBHOOK_URL`: optional. Passed as the `webhook` param when a session
  calls `create_demo_video`, so Pitch MCP can callback `/api/webhook/pitch`.
  Sessions also poll `get_job` themselves, so this is a belt-and-suspenders.
- Legacy keys (`X_API_KEY`, `X_API_SECRET`, `X_BEARER_TOKEN`, `X_USER_*`,
  `X_WEBHOOK_ID`) are unused by the code.
- Never commit `.env` or expose `PITCH_API_KEY` / OAuth2 tokens in logs.

### 3. Build & verify

```bash
cargo build --release
./target/release/pitch-cli sync                 # empty DB → zeros
./target/release/pitch-cli circuit-breaker      # expect "OK to run"
./target/release/pitch-cli budget               # confirm caps + 0 used
./target/release/pitch-cli server --port 8790 & # boot server
curl http://localhost:8790/api/webhook/health   # ok
curl "http://localhost:8790/api/webhook/x?crc_token=test"  # sha256=...
curl -X POST http://localhost:8790/api/webhook/pitch -d '{"jobId":"x","status":"COMPLETED"}'  # ok
```

Verify the MCP servers load in opencode (`opencode debug config`) and
`opencode mcp` lists `pitch` and `xmcp`; the `pitch` server needs
`PITCH_API_KEY` exported (opencode reads `{env:...}` from the process env), and
`xmcp` needs `X_CLIENT_ID`/`X_CLIENT_SECRET` and opens a one-time browser login
on first use (or run `xurl auth oauth2`).

### 4. Wire the webhook in X

Register `https://<public-url>/api/webhook/x` as the Account Activity webhook
URL in your X app. X will call `GET` (CRC) during registration and `POST` on
every mention thereafter.

## Automation checklist (end to end)

1. `.env` populated with `PITCH_API_KEY` and `X_CLIENT_ID`/`X_CLIENT_SECRET`;
   `xmcp` one-time login done; `awb up "Testing"` running.
2. `cargo build --release` passes.
3. `pitch-cli circuit-breaker` says `OK to run`; `pitch-cli budget` shows
   remaining caps.
4. `pitch-cli server` is always-on behind a public URL (dispatches sessions
   capped by `MAX_OPENCODE_SESSIONS`, logs in `data/sessions/`).
5. Optional `*/5` cron curls `/api/webhook/trigger` `{"action":"mentions"}`;
   `*/3`h cron runs `pitch-cli discover`.
6. Watch `data/sessions/*.log` + `GET /api/webhook/stats`; trip/reset the
   breaker on anomalies.
