# OAuth 1.0a User Context, X API Credit Caps & Zero-Supabase Local Architecture

## 1. OAuth 1.0a User Context vs OAuth 2.0 PKCE
- **OAuth 1.0a User Context (Never Expire):** Uses 4 keys (`X_API_KEY`, `X_API_SECRET`, `X_USER_ACCESS_TOKEN`, `X_USER_ACCESS_SECRET`) with HMAC-SHA1 signatures. Does NOT expire every 2 hours like OAuth 2.0 Bearer tokens.
- **X API v2 Free Tier Limits:** Free tier allows **500 Posts / Month** and **1,500 Reads / Month**.
- **Credit Depletion (`HTTP 402 Payment Required`):** When the monthly 500-post app quota is reached, X API returns `HTTP 402: Payment Required (credits depleted)`.
- **Automatic Fallback Net:** When `HTTP 402` or `HTTP 403` occurs on API calls, `pitch-cli` seamlessly switches to Playwright browser session (`storageState_trypitchdotco.json`), publishing posts, replies, and quotes directly on X at $0 cost.

## 2. Zero-Supabase Vercel Proxy Architecture
To completely eliminate Supabase cloud database:
1. **Local Axum HTTP Server (`port 8790`):** Exposes `/api/crm`, `/api/mentions`, `/api/stats`, `/api/insights` directly against `data/pitch_bot.db` SQLite database.
2. **HTTPS Tunnel Proxy:** Expose local port 8790 via `localtunnel --port 8790` or `cloudflared tunnel`.
3. **Vercel Serverless Handlers (`dashboard/api/`):** Update `crm.js`, `mentions.js`, `stats.js`, `insights.js`, `auth.js` to fetch directly from local Rust server URL (`LOCAL_RUST_SERVER_URL`) with `Bypass-Tunnel-Remainder: true` header.

## 3. Playwright Search Fallback & Spaced URL Normalization
When X API search endpoint returns 401/403/402, `XApiClient.search_recent` in Rust (`src/x_api.rs`) automatically switches to Playwright browser search (`https://x.com/search?q=...&f=live` using `.browser-profile/storageState.json`), extracts live founder tweets, scores ICP fit (1-10), and populates `data/pitch_bot.db`. Mention URLs with spaces (e.g. `https:// supermemory.ai`) are normalized (`https://supermemory.ai`).

## 4. Local Git Verification & Push Policy
Always test and verify code changes locally first. Do NOT execute `git push` to GitHub automatically unless explicitly requested by the user.
