# Vercel Dashboard Light Mode, Password Firewall & Manual Deletion API

This reference documents the Light Mode Vercel Dashboard UI, Password Authentication Firewall (`pitch@123`), and the manual `deleteEntry()` REST API with Pitch MCP cancellation integration.

## 1. Password Firewall (`pitch@123` / `/api/auth`)
- **Endpoint:** `POST /api/auth`
- **Password:** `pitch@123`
- **Session Mechanism:**
  - Upon submitting `pitch@123`, `/api/auth` generates an HMAC-SHA256 session token and sets an HTTP-Only cookie `pitch_auth_session`.
  - On page load, `index.html` calls `GET /api/auth`. If authenticated, the password lock screen (`#auth-overlay`) hides and reveals `#dashboard-content`.
  - If unauthenticated, the lock screen blocks access to all metrics, CRM tables, and mention job details.

## 2. Ultra-Modern Light Mode UI Design System
- **Palette:** Off-white background (`#f8fafc`), crisp white cards (`#ffffff` with `border-slate-200`), dark slate typography (`text-slate-900` headings, `text-slate-500` labels).
- **Typography:** Google Fonts `Plus Jakarta Sans` (UI) and `JetBrains Mono` (timestamps, job IDs, URLs).
- **Null-Safe DOM Rendering:** All JS DOM updates use a null-safe helper `setElText(id, text)` to guarantee zero uncaught console errors if a DOM node is omitted during filtering or layout changes.

## 3. Interactive Filters & Real-Time Search
- **Activity Log Filters:**
  - **Account Selector:** `@adnanspitch` vs `@trypitchdotco` vs `All Handles`
  - **Action Category:** `All Actions`, `Replies & Mention Demos` (`reply`, `mention_demo_reply`), `Posts & Quotes` (`post`, `quote`), `Likes` (`like`), `DMs` (`dm`)
  - **Real-Time Search Bar:** Filters activity details, handles, and timestamps dynamically as the user types.
- **Viral Mention Bot Pipeline Filters:**
  - **Status Filter:** `All Statuses`, `Delivered / Completed`, `Rendering / Processing`, `Pending / Submitted`
  - **Real-Time Search Bar:** Filters handles, target product URLs, and Pitch MCP job IDs dynamically.

## 4. Manual Deletion REST API (`POST /api/delete`)
- **Endpoint:** `POST /api/delete`
- **Supported Entity Types:**
  - `activity` $\rightarrow$ Supabase `activities` table
  - `mention_job` $\rightarrow$ Supabase `mention_jobs` table + Pitch MCP `cancel_job`/`delete_job`
  - `prospect` $\rightarrow$ Supabase `prospects` table
- **Implementation:** Uses direct dependency-free native `fetch` requests to Supabase REST API (`/rest/v1/tableName?id=eq.ID`), avoiding package bundle mismatches in Vercel serverless functions.
- **Pitch MCP Integration:** When deleting a `mention_job`:
  - Fetches current job status from Supabase `mention_jobs`.
  - If status is `rendering` or `processing`: sends `cancel_job` tool call to `https://api.trypitch.co/mcp` with `jobId`.
  - If status is `completed` or `delivered`: sends `delete_job` tool call to `https://api.trypitch.co/mcp` with `jobId`.
  - Removes entry from Supabase database table and refreshes the dashboard UI automatically.
