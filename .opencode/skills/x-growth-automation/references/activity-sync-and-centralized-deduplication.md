# Centralized Activity Sync & Serverless Deduplication Architecture

This reference documents the architecture and failure modes for syncing agent activity logs and CRM state to Supabase without accumulation of duplicate records.

## 1. Failure Mode: Unauthenticated Sync & Fallback Re-Sync
- **The Pitfall:** Local client-side sync scripts (`sync_supabase.py`) often rely on local environment variables or hardcoded API keys. If the local key is missing, truncated, or invalid (`401 Unauthorized`), unhandled exceptions or naive error fallbacks (e.g. `get_existing_activities()` returning an empty set `set()` on fetch failure) cause the client script to assume Supabase is empty.
- **The Result:** On every tick, the sync script re-uploads the entire local `activity-log.jsonl` history, creating hundreds of duplicate records in Supabase.

## 2. Serverless Centralized Sync Endpoint (`/api/sync`)
Instead of having local scripts query Supabase REST APIs directly with local keys:
1. **Server-Side Authentication:** Expose a Vercel serverless function (`/api/sync`) that utilizes `process.env.SUPABASE_SERVICE_ROLE_KEY` or `process.env.SUPABASE_ANON_KEY` injected securely at runtime in the cloud.
2. **Normalized Signature Matching:**
   Generate an activity signature for each record:
   ```javascript
   const makeSignature = (a) => {
     const ts = (a.ts || '').slice(0, 19); // Compare up to seconds
     const action = a.action || '';
     const handle = a.handle || '';
     const detail = (a.detail || '').replace(/\s+/g, ' ').trim();
     return `${ts}_${action}_${handle}_${detail}`;
   };
   ```
   Filter incoming activities against existing signatures fetched with cloud credentials before performing `POST /rest/v1/activities`.
3. **Automated Database Cleanup (`GET /api/sync`):**
   Expose a GET route on `/api/sync` that scans all rows in Supabase `activities`, detects duplicate signatures, and deletes duplicate row IDs in batches.

## 3. Client Script Standard (`sync_supabase.py`)
Local client scripts delegate sync entirely to the centralized endpoint:
```python
def sync_all():
    payload = {
        "activities": read_jsonl("state/activity-log.jsonl"),
        "prospects": read_jsonl("state/prospects.jsonl")
    }
    req = urllib.request.Request("https://dashboard-url.vercel.app/api/sync", 
                                 data=json.dumps(payload).encode("utf-8"), 
                                 headers={"Content-Type": "application/json"}, 
                                 method="POST")
    with urllib.request.urlopen(req) as resp:
        print("Sync output:", resp.read().decode("utf-8"))
```
- **Rule:** Never assume Supabase is empty if an API call fails. Abort client sync if the server endpoint returns a non-200 status code.
