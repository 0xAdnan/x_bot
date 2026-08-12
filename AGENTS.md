# PITCH X/Twitter Growth Agent — Project Rules

PITCH (https://trypitch.co) is an AI video editor that turns task descriptions into
studio-quality narrated demo MP4s. This repo is the X/Twitter growth-to-sales funnel that
promotes it: a small Rust webhook dispatcher plus an opencode skill suite that drives the
real browser and the X API v2 (via the `xmcp` MCP server).

## Architecture

- **`src/`** — `pitch-cli` (Rust, Cargo workspace bin `pitch-cli`). Modules: `server` (Axum
  webhook dispatcher, port 8790/`PORT` — receives X mention events + Pitch MCP completions
  and **dispatches an OpenCode session per event via the `opencode_rs` SDK** against the
  always-on `opencode serve` HTTP server), `discover` (ICP prospect search),
  `x_api` (internal X API v2 client, legacy/heavy path — prefer `xmcp`), `safety` (daily
  budget + circuit breaker), `db` (SQLite), `config` (env loading). The old `inbox`,
  `worker`, and `pitch_mcp` modules were deleted: the webhook dispatcher hands each event
  to an opencode session instead.
- **`data/pitch_bot.db`** — SQLite CRM + mention job queue (unless `SQLITE_DB_PATH` set).
- **`data/sessions/`** — stdout logs of every dispatched opencode session (capped by
  `MAX_OPENCODE_SESSIONS`, default 3 concurrent).
- **`opencode.jsonc`** — registers the two **MCP servers** the agent uses
  directly: `pitch` (remote `https://api.trypitch.co/mcp`, Bearer
  `PITCH_API_KEY`) and `xmcp` (official X MCP via the local `xurl` bridge to
  hosted `https://api.x.com/mcp`, OAuth2 via `CLIENT_ID`/`CLIENT_SECRET` from
  `.env`). Agent tools call these servers; `pitch-cli` no longer exposes
  `mcp`/`x-api` subcommands.
- **`.opencode/skills/x-growth/`** — orchestrator skill: boot, guardrails, session loop,
  state schemas, 13 hard rules. Shared resources: `safety.md`, `voice.md`, `learn.md`,
  `state/account.json`, `state/insights.md`.
- **`.opencode/skills/x-{prospect,engage,outreach,content,community,mention}/SKILL.md`** —
  flow skills dispatched by the orchestrator. `x-outreach/dm-templates.md` is a reference
  file. Flow skills reference shared files via project-root paths like
  `.opencode/skills/x-growth/safety.md`.
- **`.opencode/agent/x-growth.md`** — primary agent definition; declares the skills + tools.
- **`.opencode/skills/agent-webbridge/SKILL.md`** — drives the user's REAL Chrome (profile
  `Testing`, router `127.0.0.1:10086`) via agent-webbridge; full setup/install/diagnose is
  in this file (see Browser automation below). **All X writes go through webbridge**
  (`Testing`), never the internal X API client.

## Build, test & key commands

- Build: `cargo build --release` (binary `./target/release/pitch-cli`). No linter/test
  suite configured.
- Verify config/skills: `opencode debug skill`, `opencode debug config`,
  `opencode debug agent x-growth`.
- Server: `pitch-cli server` (unified webhook base `/api/webhook`: X CRC +
  mentions at `/x`, Pitch MCP completion at `/pitch`, `trigger`, `health`,
  `stats`; `--port 8790` for local runs). Each incoming event dispatches an
  OpenCode session (via the `opencode_rs` SDK → always-on `opencode serve`
  HTTP server, default `http://127.0.0.1:4096`, override `OPENCODE_URL`) that
  executes the matching skill. Start `opencode serve --port 4096` before
  `pitch-cli server`; the dispatcher health-checks it at boot.
- Safety: `pitch-cli budget` (daily caps + burst), `pitch-cli circuit-breaker` (status),
  `--trip "reason"` (pause), `--reset` (resume). Always check budget + breaker BEFORE any
  X write.
- Other commands: `pitch-cli discover` (prospect search), `pitch-cli db
  jobs|get-job|prospects|get-prospect|insights get`, `pitch-cli sync` (DB summary).
- X API: for reads, drive the `xmcp` MCP server tools (me/lookup/search/
  mentions) callable from opencode. **For writes (post/reply/like), always use
  agent-webbridge (`Testing` profile)**, not the internal X API client.
- Pitch MCP: call the `pitch` MCP server tools (`create_demo_video`, `get_job`,
  `get_credits`) in opencode; `PITCH_WEBHOOK_URL` is optional now — dispatched
  sessions poll `get_job` themselves rather than relying on the `/pitch` callback.

## Operating rules (hard)

1. **Account identity**: acting account is `@trypitchdotco`. Brand claims/voice:
   `.opencode/skills/x-growth/voice.md`. Only approved claims; never invent data.
2. **Safety limits**: `.opencode/skills/x-growth/safety.md` caps (likes ~≤60/day, replies
   ~≤30/day, DMs ~≤15/day, burst spacing ≥90s) are hard limits — never exceed.
3. **Check breaker + budget first**: any X write pass starts with `pitch-cli budget` and
   `pitch-cli circuit-breaker`. If breaker is tripped or budget exhausted, STOP.
4. **Dry-run first**: use `--dry` on post/reply/like and pipeline commands before real
   execution.
5. **CRM discipline**: log every action to SQLite (`db log`, `db upsert-prospect`) and keep
   `state/prospects.jsonl`/`activity-log.jsonl`/`insights.md` current; run the retro in
   `learn.md`.
6. **Security**: `.env` is gitignored and must NOT be committed. `PITCH_API_KEY` and OAuth2
   tokens are secrets — never log or expose them. Do not print `.env` contents.

## Boot guardrail order (x-growth skill)

Verify in this exact order each session: (1) operator/account identity + persona → (2)
voice & claims compliance → (3) safety caps + burst pacing → (4) budget + circuit breaker
status → (5) state files (account.json, insights.md, prospects/activity logs) → (6)
skill/tool availability.

## `.env` requirements (gitignored)

Required vars (see `src/config.rs`): `X_CLIENT_ID`, `X_CLIENT_SECRET`,
`X_OPERATOR_HANDLE`, `X_USERNAME`, `X_PASSWORD`, `PITCH_API_KEY`, optional
`SQLITE_DB_PATH`, `PITCH_WEBHOOK_URL` (public URL of `/api/webhook/pitch` on the
webhook server), `X_WEBHOOK_ID`, `MAX_OPENCODE_SESSIONS` (default 3), optional
`OPENCODE_URL` (OpenCode server base URL, default `http://127.0.0.1:4096`).
Legacy
keys `X_API_KEY`/`X_API_SECRET`/`X_BEARER_TOKEN` and the OAuth2 `X_USER_*`
tokens are unused by the code (the internal `x_api` client is off-limits).
`X_CLIENT_ID`/`X_CLIENT_SECRET` feed the `xmcp` xurl bridge OAuth2 login.

## Browser automation (agent-webbridge)

Full setup/install/diagnose flow lives in this repo's `.opencode/skills/agent-webbridge/`.
Always run `awb status` first and act on its result. In THIS project use profile
`Testing` (never the docs' `Work`/`Personal` examples); router on `127.0.0.1:10086`.
Quick start: `npm i -g agent-webbridge`, `awb setup "Testing"`, `awb connect "Testing"`,
`awb up "Testing"`, `awb status`, `awb down` to tear down.

## Gotchas

- **X MCP auth via `xmcp`**: the OAuth2 user tokens in `.env`
  (`X_USER_ACCESS_TOKEN`/`X_USER_REFRESH_TOKEN`) are expired/invalid for the old
  `x-api` client. The `xmcp` MCP server (official X MCP through the `xurl`
  bridge) uses its own OAuth2 PKCE login cached in `~/.xurl`; first run opens the
  browser (or run `xurl auth oauth2` once). X reads go through `xmcp`; X writes
  go through agent-webbridge (`Testing` profile) and are NOT blocked. The single
  remaining internal X call is `discover` — blocked until `.env` has fresh tokens.
