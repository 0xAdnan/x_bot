# Pitch MCP API Integration Details (`https://api.trypitch.co/mcp`)

## MCP Server Endpoint & Auth
- **URL:** `https://api.trypitch.co/mcp`
- **Headers:**
  - `Authorization: Bearer <PITCH_API_KEY>`
  - `Content-Type: application/json`
  - `Accept: application/json, text/event-stream`
  - `User-Agent: Mozilla/5.0 (X11; Linux x86_64)` *(Mandatory to bypass Cloudflare 403)*

## Supported MCP Tools

### 1. `create_demo_video`
- **Description:** Creates an AI demo video job for a product URL (costs 3 credits).
- **Full Argument Schema:**
```json
{
  "url": "https://tella.com",
  "instructions": "Create a cinematic, polished product demo of https://tella.com. Highlight key features, value proposition, and user experience.",
  "voice": "Charon",
  "subtitles": false,
  "theme": "light",
  "background": "ocean",
  "shape": "rounded",
  "inset": "0.75",
  "browserHeader": "light"
}
```
- **MCP Tool Call Payload:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_demo_video",
    "arguments": {
      "url": "https://tella.com",
      "instructions": "Create a cinematic, polished product demo of https://tella.com.",
      "voice": "Charon",
      "subtitles": false,
      "theme": "light",
      "background": "ocean",
      "shape": "rounded",
      "inset": "0.75",
      "browserHeader": "light"
    }
  }
}
```
- **Returns:** `{ "jobId": "cmsm744z70009sc4dww5vb2yl", "status": "PENDING" }`

### 2. `get_job`
- **Description:** Get job progress, phases, and rendered video S3 URL.
- **Payload:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "get_job",
    "arguments": { "jobId": "cmsm744z70009sc4dww5vb2yl" }
  }
}
```
- **Returns:**
```json
{
  "id": "cmsm744z70009sc4dww5vb2yl",
  "status": "PROCESSING",
  "progress": 5,
  "phases": [
    { "phase": "workspace_init", "status": "completed" },
    { "phase": "video_recording", "status": "running" }
  ]
}
```

### 3. `get_credits`
- **Description:** Get API owner credit balance and transaction history.
- **Payload:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": { "name": "get_credits", "arguments": {} }
}
```

## Dashboard Job Deletion & Cancellation API Integration (`cancel_job` vs `delete_job`)
When a user deletes a mention job entry from the live Vercel dashboard:
1. **In-Flight Jobs (`rendering` / `processing` / `pending`):** The dashboard API (`/api/delete`) sends a `cancel_job` tool call to `https://api.trypitch.co/mcp` with `{ "jobId": "..." }` to stop active browser rendering and refund/preserve credits.
2. **Finished Jobs (`completed` / `delivered`):** The dashboard API sends a `delete_job` tool call to `https://api.trypitch.co/mcp` with `{ "jobId": "..." }` to clean up server-side job records.
3. **Database Sync:** The entry is then deleted from Supabase `mention_jobs` table and the live dashboard UI refreshes instantly.

