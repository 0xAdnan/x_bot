# X Webhook CRC Challenge & API Tier Nuances

## X Account Activity API / Webhook CRC Challenge
When registering a webhook URL (e.g. `https://dashboard-blue-five-75.vercel.app/api/x-webhook`) in X Developer Portal:

1. **CRC Trigger:** X sends a `GET` request with `crc_token`.
2. **HMAC Calculation:** The server must calculate `HMAC-SHA256(crc_token, Consumer_Secret)` base64-encoded.
   * **CRITICAL PITFALL:** The HMAC calculation **MUST** use the app's **Consumer Secret (API Secret Key `X_API_SECRET`)**, NOT the OAuth 2.0 Client Secret (`X_CLIENT_SECRET`). Using `X_CLIENT_SECRET` causes X Developer Portal to reject the response with `Invalid response_token`.
3. **Response Payload:**
```json
{
  "response_token": "sha256=BASE64_ENCODED_HMAC_DIGEST"
}
```

## X API v2 Free Tier vs Paid Tier Nuances
- **`POST /2/tweets` (Publishing / Replying):** Included for free on Free Tier (up to 1,500 tweets/month).
- **`GET /2/tweets/search/recent` or `GET /2/users/:id/mentions` (Reading Mentions):** Returns `402 Payment Required` (`detail: credits depleted`) on Free Tier.
- **Hybrid Solution:** Scan mentions using browser automation on `https://x.com/search?q=%40trypitchdotco&f=live` (unfiltered, free, real-time) or Native Webhooks (`POST /api/x-webhook`), and publish replies via official API (`POST /2/tweets`).
