# SQLite to Vercel/Supabase Dashboard Live Sync Architecture

This reference documents the live synchronization mechanism that mirrors local SQLite database memory (`data/pitch_bot.db`) on the `rewrite/clean-slate` Rust branch directly to the visual Vercel Web Dashboard (`https://dashboard-blue-five-75.vercel.app`) and Supabase database.

## 1. Dual Storage Architecture
- **Local Engine (Sub-Millisecond Core):** The Rust `pitch-cli` binary reads and writes directly to local SQLite (`data/pitch_bot.db`). This provides sub-millisecond execution speeds, offline durability, and zero API auth latency during agent sessions.
- **Visual Web Dashboard (Vercel + Supabase):** The human-facing visual UI (`https://dashboard-blue-five-75.vercel.app`) displays CRM Kanban pipelines, video render links, and live activity feeds backed by Supabase.

## 2. Background Sync Daemon (`bin/sync_sqlite_to_dashboard.py`)
To keep the Vercel Web Dashboard perfectly synchronized with local Rust agent actions:

- A background daemon (`bin/sync_sqlite_to_dashboard.py`) executes a lightweight 10-second sync loop.
- It queries local SQLite tables (`mention_jobs`, `prospects`, `activities`).
- It POSTs the JSON payload to `https://dashboard-blue-five-75.vercel.app/api/sync`.

```python
# Background Sync Daemon Loop
def do_sync():
    conn = sqlite3.connect("data/pitch_bot.db")
    conn.row_factory = sqlite3.Row
    c = conn.cursor()

    c.execute("SELECT * FROM mention_jobs")
    jobs = [dict(r) for r in c.fetchall()]

    c.execute("SELECT * FROM prospects")
    prospects = [dict(r) for r in c.fetchall()]

    c.execute("SELECT * FROM activities ORDER BY id DESC LIMIT 50")
    activities = [dict(r) for r in c.fetchall()]

    payload = json.dumps({
        "mention_jobs": jobs,
        "prospects": prospects,
        "activities": activities
    }).encode("utf-8")

    req = urllib.request.Request("https://dashboard-blue-five-75.vercel.app/api/sync", data=payload, headers={"Content-Type": "application/json"}, method="POST")
    with urllib.request.urlopen(req) as resp:
        pass
```

## 3. Server-Side Signature Deduplication (`/api/sync`)
- On the Vercel serverless side, `/api/sync` validates and normalizes signatures (`ts + action + handle + cleanDetail`) for activities, and primary keys for `mention_jobs` (`tweet_id`) and `prospects` (`handle` / `product_url`).
- This guarantees zero duplicate row accumulation in Supabase even when the sync daemon posts every 10 seconds.
