# PITCH X/Twitter Growth Agent — Project Rules

PITCH (https://trypitch.co) is an AI video editor that turns task descriptions into
studio-quality narrated demo MP4s. This repo is the X/Twitter growth-to-sales funnel that
promotes it: a pure-Rust CLI/webhook pipeline plus an opencode skill suite that drives the
real browser and the X API v2.

## Architecture

- **`src/`** — `pitch-cli` (Rust, Cargo workspace bin `pitch-cli`). Modules: `server` (Axum
  webhook server, port 8080/`PORT`), `inbox` (mention ingestion), `worker` (demo-render
  delivery), `discover` (ICP prospect search), `x_api` (X API v2 client with OAuth2
  auto-refresh), `safety` (daily budget + circuit breaker), `pitch_mcp` (Pitch video MCP),
  `db` (SQLite), `config` (env loading).
- **`data/pitch_bot.db`** — SQLite CRM + mention job queue (unless `SQLITE_DB_PATH` set).
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
  in this file (see Browser automation below).

## Build, test & key commands

- Build: `cargo build --release` (binary `./target/release/pitch-cli`). No linter/test
  suite configured.
- Verify config/skills: `opencode debug skill`, `opencode debug config`,
  `opencode debug agent x-growth`.
- Server: `pitch-cli server` (health `GET /api/webhook/health`, CRC
  `/api/webhook/x?crc_token=...`, `pitch-cli server --port 8790` for local runs).
- Safety: `pitch-cli budget` (daily caps + burst), `pitch-cli circuit-breaker` (status),
  `--trip "reason"` (pause), `--reset` (resume). Always check budget + breaker BEFORE any
  X write.
- Pipeline (all `--dry` first): `pitch-cli trigger`, `inbox`, `worker`, `discover`,
  `sync` (DB summary).
- DB: `pitch-cli db jobs|get-job|prospects|get-prospect|insights get`.
- X API: `pitch-cli x-api me|lookup|search|mentions|post|reply|like|refresh`
  (post/reply/like support `--dry`; writes currently 401 — see Gotchas).
- Pitch MCP: `pitch-cli mcp create <url> [instructions]|status <job_id>|credits`.

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
`X_USER_ACCESS_TOKEN`, `X_USER_REFRESH_TOKEN`, `X_USER_ID`, `X_OPERATOR_HANDLE`,
`X_USERNAME`, `X_PASSWORD`, `PITCH_API_KEY`, optional `SQLITE_DB_PATH`, `X_WEBHOOK_ID`. Legacy keys
`X_API_KEY`/`X_API_SECRET`/`X_BEARER_TOKEN` are unused by the code.

## Browser automation (agent-webbridge)

Full setup/install/diagnose flow lives in this repo's `.opencode/skills/agent-webbridge/`.
Always run `awb status` first and act on its result. In THIS project use profile
`Testing` (never the docs' `Work`/`Personal` examples); router on `127.0.0.1:10086`.
Quick start: `npm i -g agent-webbridge`, `awb setup "Testing"`, `awb connect "Testing"`,
`awb up "Testing"`, `awb status`, `awb down` to tear down.

## Gotchas

- **X API v2 401**: OAuth2 user tokens in `.env` are expired/invalid; `x-api me` fails
  even after `refresh`. Any X API write flow (x-mention, discover via API) is blocked until
  fresh tokens are saved to `.env`. Browser-based actions via webbridge are NOT blocked.
- **Hardcoded key**: `src/config.rs` falls back to a real `PITCH_API_KEY` if unset — do not
  rely on that fallback; always supply it via `.env`.
