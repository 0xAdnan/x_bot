# X API Credentials, Clerk Auth & Supabase Deduplication Notes

## 1. Official X API Credentials & Capability
- **App Credentials:** Client ID `bWxmMm1LWXF0ZDR0YWVsek9EdkQ6MTpjaQ`, Client Secret `Q8d7SpSo-RoKLaZBDDz00aEbt59cJ9Yr44SeVPsR2X8D6g9i3D`
- **User Tokens (`@trypitchdotco`):** Access Token `WThaYmhud0VoU0l1SnkyblVBQUFrQlJzZGFWWUtkWVo3QUI4azRIM3hMektNOjE3ODYyOTg4NTU4MzM6MTowOmF0OjE`
- **Free Tier Read Restriction:** `GET /2/users/:id/mentions` returns `402 Payment Required` (credits depleted).
- **Hybrid Solution:**
  - Read/Scan: Browser automation via Playwright on `x.com/search?q=%40trypitchdotco&f=live` (unfiltered, free).
  - Write/Reply: Official API via `bin/xurl-pitch -X POST /2/tweets`.

## 2. trypitch.co Clerk Authentication
- Clerk uses short-lived 60-second access JWTs (`__session`) and long-lived 1-year refresh tokens (`__client`).
- The `__client` cookie value starting with `eyJhbGciOiJSUzI1NiIs...` must be present in `storageState_trypitch_co.json` so Playwright browser automation never redirects unauthenticated from `trypitch.co/new`.
- **Form Selectors on `trypitch.co/new`:**
  - Tab: "Website demo"
  - Inputs: Product URL, Demonstration Prompt
  - Presets: Audio "Charon", Header "Light", Background "Ocean"

## 3. Supabase Activity & Job Deduplication
- Sync script `sync_supabase.py` pre-fetches existing `(ts, action, detail)` tuples before sending POST requests to prevent duplicate row creation.
- SQL cleanup query for `activities` table:
  ```sql
  DELETE FROM public.activities a
  WHERE a.id NOT IN (
      SELECT MIN(id) FROM public.activities GROUP BY ts, action, detail
  );
  ```
