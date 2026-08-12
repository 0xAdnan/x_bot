# Git Push Policy & Mention URL Normalization

## 1. Local-First Git Commit & Push Policy
- **User Preference:** Code changes, bug fixes, and feature implementations MUST be committed and verified locally first.
- **Explicit Push Rule:** Do NOT automatically run `git push` to GitHub (`origin/main` or feature branches) unless the user explicitly directs you to push in that turn.
- **Ad-Hoc Local Verification:** Always run local test scripts or compilation checks (`cargo check`, `node -c`, python test scripts) to produce passing verification evidence before marking work as complete.

## 2. Space-Tolerant URL Normalization in Mention Tweets
- **Pattern:** Users frequently type URLs with spaces inside tweet text (e.g. `https:// supermemory.ai` or `http:// zerith.studio`).
- **Standard Regex Failure:** Naive regexes like `https?://[^\s]+` fail on spaced URLs and silently drop valid mentions.
- **Normalized Regex:** Use space-tolerant capture groups:
  ```python
  re_http = re.compile(r"https?://\s*([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?)")
  ```
  ```rust
  let re_http = Regex::new(r"https?://\s*([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?)").unwrap();
  ```
- **Domain Cleaning:** Strip leading/trailing spaces and prepend `https://` if missing.

## 3. OAuth 2.0 PKCE Client ID Pairing Rule
- **Client ID Matching:** In X API v2 OAuth 2.0 PKCE token refresh (`POST /2/oauth2/token`), X requires the `client_id` parameter to match the exact X Developer Portal App that issued the `refresh_token`.
- **Error Signal:** If `X_CLIENT_ID` in `.env` belongs to a different app than the refresh token issuer, X API returns:
  `{"error":"invalid_client","error_description":"Value passed for the client id was invalid."}`
- **Fallback Strategy:** When official API tokens expire or fail authentication, fall back seamlessly to authenticated Playwright browser sessions (`storageState_trypitchdotco.json`).

## 4. Playwright Browser Search Fallback Pattern
- **API Error Signal:** When X API v2 `/tweets/search/recent` returns `401 Unauthorized` or `403 Unsupported Authentication` due to access token expiry, direct REST search fails and returns 0 prospects.
- **Fallback Execution:** Instead of returning empty results, `XApiClient.search_recent` automatically triggers a Playwright browser search pass:
  1. Open browser context with active profile session (`.browser-profile/storageState.json`).
  2. Navigate to `https://x.com/search?q={encoded_query}&f=live`.
  3. Extract live tweet elements (`article[data-testid="tweet"]`), parse handles, clean text, and score ICP fit.
  4. Return discovered prospects to the CRM pipeline.
