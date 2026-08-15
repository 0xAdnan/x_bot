# Zero-Supabase Local SQLite Architecture & Direct Axum Proxying

## 1. Overview
This reference specifies how to completely eliminate cloud Supabase dependency while maintaining a live remote Vercel Web Dashboard. All pipeline state, CRM leads, and mention jobs live in local SQLite (`data/pitch_bot.db`).

---

## 2. Architectural Data Flow

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           LOCAL SYSTEM (Your Local Environment)                         │
│                                                                                         │
│   Rust Engine (`pitch-cli`) ───► Local SQLite Database (`data/pitch_bot.db`)             │
│                                           │                                             │
│                                           ▼                                             │
│                        Embedded Axum Server (`pitch-cli server` - Port 8790)            │
│                                           │                                             │
│                                           ▼                                             │
│                        HTTPS Tunnel Proxy (`localtunnel` / `cloudflared`)              │
└───────────────────────────────────────────┬─────────────────────────────────────────────┘
                                            │
                                            │ HTTPS JSON
                                            ▼
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                             CLOUD VERCEL WEB DASHBOARD                                  │
│                     (https://dashboard-blue-five-75.vercel.app)                         │
│                                                                                         │
│   • Password Auth (`pitch@123`) ───► Local Password Validation                         │
│   • CRM Kanban Pipeline         ───► Proxies to `http://localhost:8790/api/crm`        │
│   • Mention Jobs Pipeline       ───► Proxies to `http://localhost:8790/api/mentions`   │
│   • Agent System Health Monitor ───► Proxies to `http://localhost:8790/api/stats`      │
│   • Memory Insights             ───► Proxies to `http://localhost:8790/api/insights`   │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Vercel Serverless Proxy Endpoints (`dashboard/api/`)

To eliminate Supabase while keeping the visual Vercel Web UI, all serverless handlers in `dashboard/api/` proxy directly to the local Axum server URL (`LOCAL_RUST_SERVER_URL` or tunnel fallback):

1. **`dashboard/api/crm.js`**:
   Proxies `GET /api/crm` to local Rust server, returning prospects grouped by stage (`new`, `warming`, `contacted`, `in_convo`, `trial`, `customer`, `do-not-contact`, `lost`).
2. **`dashboard/api/mentions.js`**:
   Proxies `GET /api/mentions` to local Rust server, returning all `mention_jobs` from `data/pitch_bot.db`.
3. **`dashboard/api/stats.js`**:
   Proxies `GET /api/stats` to local Rust server, returning system agent health & daemon heartbeat metrics.
4. **`dashboard/api/insights.js`**:
   Proxies `GET /api/insights` to local Rust server, returning adaptive memory notes.
5. **`dashboard/api/auth.js`**:
   Validates `pitch@123` password locally, sets `pitch_auth` HTTP-Only cookie, and returns `{ authenticated: true, status: 'ok', token }`.

---

## 4. Playwright Search Fallback for X API 401/403 Errors

When the official X API v2 search endpoint (`/tweets/search/recent`) fails due to OAuth 2.0 PKCE scope mismatch or 401/403 authorization errors:

1. `XApiClient.search_recent` in `src/x_api.rs` detects the API failure.
2. It automatically executes a Playwright browser search fallback using python:
   `https://x.com/search?q={query}&f=live` with session cookies (`.browser-profile/storageState.json`).
3. Extracts live tweets, scores ICP fit (1–10), auto-generates pitch hooks, and saves discovered leads into `data/pitch_bot.db`.

---

## 5. Tiled 4-Pane Tmux Dashboard (`pitch-dashboard`)

To view all agents simultaneously in a single split-screen window:

```bash
bash /home/adnan/x_bot/bin/start_tmux_dashboard.sh
tmux attach -t pitch-dashboard
```

Layout:
- **Pane 0:** Axum Webhook Server (`pitch-cli server --port 8790`)
- **Pane 1:** 10s Trigger Loop (`rust_trigger_loop.py`)
- **Pane 2:** SaaS Prospect Discovery Agent (`pitch-cli discover --max 5`)
- **Pane 3:** 24/7 AI Research Daemon (`rust_researcher_loop.py`)
