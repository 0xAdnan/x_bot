# Zero-Supabase Architecture Specification

## Overview

The PITCH X Bot system has been transitioned from a cloud-dependent Supabase infrastructure to a **100% Zero-Supabase Local SQLite Architecture**. 

All pipeline operations, lead CRM state, mention jobs, and agent activities run locally on disk inside a single SQLite database (`data/pitch_bot.db`). The Vercel Web Dashboard queries the local SQLite database directly via a secure HTTPS tunnel proxying to the embedded Rust Axum HTTP server (`pitch-cli server`).

---

## Architectural Diagram

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
│                        HTTPS Tunnel Proxy (`fruity-corners-crash.loca.lt`)             │
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

## Key Components & Responsibilities

### 1. Local Rust Engine (`pitch-cli`)
* **Location:** `/home/adnan/x_bot/target/release/pitch-cli`
* **Role:** Single compiled executable handling all core operations:
  * Ingests incoming X mentions & extracts target product URLs (handling space-formatted URLs).
  * Submits video rendering jobs to Pitch MCP (`create_demo_video`).
  * Discovers SaaS founder prospects on X (`search_recent` with Playwright fallback).
  * Enforces safety budget limits and account circuit breakers.

### 2. Local SQLite Memory (`data/pitch_bot.db`)
* **Location:** `/home/adnan/x_bot/data/pitch_bot.db`
* **Role:** Single source of truth for all pipeline state:
  * **`mention_jobs` Table:** Tracks tweet IDs, handles, URLs, Pitch MCP job IDs, render status, and final S3 video links.
  * **`prospects` Table:** Tracks CRM pipeline stages (`new`, `warming`, `contacted`, `in_convo`, `customer`), fit scores (1–10), and pitch hooks.
  * **`activities` Table:** Logs likes, replies, and discovery actions with timestamps for safety enforcement.
  * **`insights` Table:** Stores adaptive agent memory and conversion notes.

### 3. Embedded Axum Webhook Server (`src/server.rs`)
* **Location:** `http://0.0.0.0:8790`
* **Role:** Embedded Rust HTTP server providing REST endpoints directly against `data/pitch_bot.db`:
  * `GET /api/crm`: Serves CRM prospects grouped by stage.
  * `GET /api/mentions`: Serves all mention jobs.
  * `GET /api/stats`: Serves pipeline statistics & active agent count.
  * `GET /api/insights`: Serves adaptive memory notes.
  * `GET/POST /api/webhook/x`: Handles real-time X webhook callbacks and CRC challenges.

### 4. Vercel Serverless Proxy Layer (`dashboard/api/`)
* **Role:** Serves the visual web UI on `https://dashboard-blue-five-75.vercel.app` without Supabase:
  * **`dashboard/api/crm.js`**: Proxies `/api/crm` to local Rust server.
  * **`dashboard/api/mentions.js`**: Proxies `/api/mentions` to local Rust server.
  * **`dashboard/api/stats.js`**: Proxies `/api/stats` to local Rust server.
  * **`dashboard/api/insights.js`**: Proxies `/api/insights` to local Rust server.
  * **`dashboard/api/auth.js`**: Validates `pitch@123` password and sets HTTP-Only auth cookies.

---

## API Endpoint Mapping

| Vercel UI Endpoint | Proxy Destination | SQLite Table Queried | Purpose |
| --- | --- | --- | --- |
| **`GET /api/crm`** | `http://localhost:8790/api/crm` | `prospects` | Renders CRM Kanban columns |
| **`GET /api/mentions`** | `http://localhost:8790/api/mentions` | `mention_jobs` | Renders Mention Video Jobs table |
| **`GET /api/stats`** | `http://localhost:8790/api/stats` | `activities` & `mention_jobs` | Renders Agent Health Monitor |
| **`GET /api/insights`** | `http://localhost:8790/api/insights` | `insights` | Renders Adaptive Strategy Insights |
| **`POST /api/auth`** | Local Auth Handler | N/A | Password authentication (`pitch@123`) |

---

## Benefits of the Zero-Supabase Architecture

1. **Sub-Millisecond Performance:** All database reads and writes execute against local SQLite in < 1ms, eliminating network latency.
2. **Zero Third-Party Cloud Failures:** No dependency on Supabase REST API connection strings, secret key truncation, or cloud downtime.
3. **100% Privacy & Control:** Your CRM prospects, mention logs, and pitch notes remain on your local disk.
4. **Single Source of Truth:** `data/pitch_bot.db` is the only database in the entire system.
