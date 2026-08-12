# Real-Time Agent System Health & Daemon Monitor Architecture

This reference defines the real-time background agent and daemon health monitoring system integrated into the Vercel dashboard and Supabase API backend.

## 1. Background Agent & Daemon Roster

The system comprises 5 distinct autonomous agents and background processes:

| Agent / Daemon | Executable / Script | Type | Target / Role | Status Indicator |
| --- | --- | --- | --- | --- |
| **Mention Scanner** | `bin/mention_daemon.py` | 10s Real-Time Poller | `x.com/notifications` + `/search` | `ACTIVE` (Continuous 10s Loop) |
| **Pitch MCP Worker** | `bin/mention_mcp_worker.py` | Queue & Video Generator | Supabase $\leftrightarrow$ Pitch MCP | `ACTIVE` (Continuous 10s Loop) |
| **Hermes Gateway** | `hermes gateway run` | Scheduler Gateway Daemon | Background Agent Engine | `ACTIVE` (Daemon Gateway) |
| **Brand & Content** | Cron Job `1b0efb03b043` | 3x Daily Cron Job | `@trypitchdotco` | `SCHEDULED` (0 8,11,14,17,20,23 * * *) |
| **Operator Growth** | Cron Job `8af37af0d747` | 3x Daily Cron Job | `@adnanspitch` | `SCHEDULED` (0 9,14,19 * * *) |

## 2. API Endpoint Architecture (`/api/stats.js`)

`/api/stats.js` serves live agent health metadata, background process status, active counts, and metrics:

```json
{
  "status": "ok",
  "timestamp": "2026-08-10T19:08:50.225Z",
  "agents": [
    {
      "id": "mention_daemon",
      "name": "Mention Scanner",
      "type": "10s Real-Time Poller",
      "target": "x.com/notifications + /search",
      "status": "active",
      "uptime": "Continuous 10s Loop",
      "last_pulse": "2026-08-10T19:08:50.225Z",
      "description": "Scans notifications & search every 10s for @trypitchdotco mentions."
    },
    {
      "id": "mention_mcp_worker",
      "name": "Pitch MCP Worker",
      "type": "Queue & Video Generator",
      "target": "Supabase <-> Pitch MCP API",
      "status": "active",
      "uptime": "Continuous 10s Loop",
      "last_pulse": "2026-08-10T19:08:50.225Z",
      "description": "Claims pending jobs, triggers Pitch MCP rendering, and posts X replies."
    },
    {
      "id": "hermes_gateway",
      "name": "Hermes Gateway",
      "type": "Scheduler Daemon",
      "target": "Background Agent Engine",
      "status": "active",
      "uptime": "Active Gateway",
      "last_pulse": "2026-08-10T19:08:50.225Z",
      "description": "Triggers scheduled growth sessions, content posting, and cron jobs."
    },
    {
      "id": "brand_agent",
      "name": "Brand & Content",
      "type": "3x Daily Cron Job",
      "target": "@trypitchdotco",
      "status": "scheduled",
      "next_run": "Today at 08:00 AM IST",
      "schedule": "0 8,11,14,17,20,23 * * *",
      "description": "Publishes product demos and founder commentary for @trypitchdotco."
    },
    {
      "id": "operator_agent",
      "name": "Operator Growth",
      "type": "3x Daily Cron Job",
      "target": "@adnanspitch",
      "status": "scheduled",
      "next_run": "Today at 09:00 AM IST",
      "schedule": "0 9,14,19 * * *",
      "description": "Discovers SaaS founders, warm-up likes, and non-promotional replies."
    }
  ],
  "total_agents": 5,
  "active_daemons_count": 3,
  "scheduled_cron_count": 2
}
```

## 3. Dashboard UI Component (`renderAgents`)

The dashboard UI renders dark-themed agent status cards directly in the top header grid, featuring live pulsing status badges (`ACTIVE` vs `SCHEDULED`), target descriptions, and 10s automatic refresh cycles.

## 4. Dynamic Heartbeat Detection & Process Failure Handling

To ensure status badges reflect real-time process health (rather than static defaults):

1. **Heartbeat Emission:** Running daemons (`bin/mention_daemon.py` and `bin/mention_mcp_worker.py`) periodically send heartbeat pulses (`action: 'heartbeat'`) to `/api/sync` on every loop cycle (every 10s).
2. **Server-Side Expiry Threshold (`/api/stats`):** When `/api/stats` runs, it queries Supabase `activities` for the latest heartbeat per daemon segment. If `nowEpoch - lastHeartbeatTs > 60000` (60 seconds), the daemon process is detected as stopped/offline.
3. **Live UI Transition:** The agent status automatically transitions from `ACTIVE` (emerald pulsing dot) to `STOPPED` (amber/red indicator: `Offline - Daemon process is stopped`), giving operators instant visibility if a process is killed or crashes.
