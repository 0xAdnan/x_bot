# Mention Bot Deduplication, Dual-Scan & Universal Idempotency Architecture

This reference outlines the exact deduplication mechanics, dual-scan ingestion, spaced URL handling, universal idempotency across all post types, and event-driven queue architecture across multi-tier mention and content automation setups.

## 1. Non-Deterministic Keying Pitfall (`hash(text)`)
In Python, calling `hash("string")` uses a process-level randomized seed (`PYTHONHASHSEED`). 
- **The Bug:** If state keys are saved as `f"{user_handle}_{hash(text)}"`, every time the process or daemon restarts, `hash(text)` evaluates to a different integer. Loaded keys from `processed_tweets.json` fail to match, causing every tweet in the feed to be re-processed and issuing duplicate API calls / video rendering jobs.
- **The Fix:**
  - Extract the real numeric `tweet_id` from Playwright/DOM tweet status links (`a[href*="/status/"]`).
  - Fall back to deterministic hashing: `hashlib.sha256(f"{user_handle}_{text[:100]}".encode('utf-8')).hexdigest()[:18]`.

## 2. Dual-Scan Ingestion (`notifications` + `search`) & Spaced URL Normalization
- **Search Omission Pitfall:** Scanning `x.com/search?q=%40handle&f=live` alone misses mentions because X search indexing delays or suppresses low-authority tweets.
- **Notifications Ingestion:** Always scan `https://x.com/notifications` alongside `x.com/search` to capture 100% of direct user mentions in real-time.
- **Remove Truncation Slices:** Never slice feed elements with artificial limits like `tweets[:5]`. Scan up to 20 visible tweets per cycle to handle mention bursts.
- **Spaced URL Handling:** Users often type URLs with spaces (e.g. `https:// supermemory.ai`). Always strip whitespace after `https://` / `http://` or normalize spaced URLs (`https:// supermemory.ai` -> `https://supermemory.ai`) before running domain validation and MCP triggering.
- **Client ID Base64 Decoding in Token Refresh:** When `.env` stores base64-encoded `X_CLIENT_ID` (e.g. `bWxmMm1...`), token refresher scripts (`bin/refresh_x_token.py`) must attempt `try_b64decode` to extract the raw Client ID (`mlf2mK...`) required by Twitter's OAuth 2.0 endpoint (`/2/oauth2/token`).
- **Track 100% of Mentions:** Mentions without a product URL should not be discarded silently. Log them to Supabase `mention_jobs` with `status: 'no_url_found'` and `target_url: 'N/A'` so operators can see all incoming mentions in the dashboard.

## 3. Event-Driven Bi-Directional Queue Architecture (`mention_mcp_worker.py`)
Rather than having multiple daemons trigger external rendering APIs independently, use Supabase as a central state broker with a single bi-directional worker:

```
[X Webhook / Dual-Scan Daemon]
             │ Writes new mention (status: 'pending')
             ▼
   [Supabase `mention_jobs`]
             │ Single Source of Truth (UNIQUE tweet_id)
             ▼
[Worker Daemon (`mention_mcp_worker.py`)]
   ├── Direction A: Claims `pending` -> Calls Pitch MCP -> Sets `status: 'rendering'`
   └── Direction B: Polls `rendering` -> Extracts S3 MP4 -> Posts X Reply -> Sets `status: 'delivered'`
```

## 4. Official X API Priority, Auto Token Refresher & Playwright Fallback
To avoid account blocks or spam detection associated with browser automation:
- **Official X API Priority:** Always execute all original posts, quote tweets, and mention replies via official X API v2 endpoints (`Authorization: Bearer X_USER_ACCESS_TOKEN` / `xurl-pitch`) first.
- **Hourly Token Refresher (`bin/refresh_x_token.py`):** Automatically exchanges OAuth 2.0 refresh tokens every 45–60 minutes at `https://api.twitter.com/2/oauth2/token` to maintain fresh access tokens in `.env`.
- **Secondary Browser Fallback:** Playwright browser automation (`storageState_trypitchdotco.json`) is strictly a secondary safety net used only when official API calls return unrecoverable HTTP 401/403/429 errors.
- **Single Worker Process Pattern:** Keep a single dedicated background worker process per automated pipeline (`bin/mention_mcp_worker.py`) to handle both request claiming and delivery state transitions.
- **Mention Jobs:** Primary key `tweet_id` (e.g. `2086533777963270217_supermemory`) with Supabase `UNIQUE(tweet_id)`.
- **Activities (Posts, Quotes, Replies):** Serverless `/api/sync` normalizes signatures (`YYYY-MM-DD` + `action` + `handle` + `cleanDetail`) before inserting to Supabase `activities`.

## 5. Direct Tweet Logging & Dashboard Link
To allow manual inspection by operators:
- **Tweet URL Construction:** `https://x.com/[clean_handle]/status/[tweet_id]`
- **Console Log Standard:**
  ```
  ========================================================================
  [MENTION BOT DETECTED NEW TWEET]
  Requested By: @user
  Mention Tweet URL: https://x.com/user/status/2086520813185044691
  Tweet Text: "Hey @trypitchdotco check out https://tella.com"
  Target Product URL: https://tella.com
  ========================================================================
  ```
- **Dashboard Table Display:** Include a dedicated column `"Mention Tweet"` rendering a clickable button:
  `<a href="https://x.com/user/status/2086520813185044691" target="_blank">View Tweet</a>`
