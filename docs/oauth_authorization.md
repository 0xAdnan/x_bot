# X OAuth2 Authorization (via `xurl` / the `xmcp` MCP server)

The old `pitch-cli x-api authorize` flow was removed. X operations now run
through the official **X MCP** server (`xmcp` in `opencode.jsonc`), reached via
the first-party **`xurl`** local bridge that owns the OAuth2 PKCE login and
auto-refreshes tokens.

---

## Overview

`xmcp` is a local stdio MCP server: `npx -y @xdevplatform/xurl mcp
https://api.x.com/mcp`. On first use with no cached token, the bridge opens the
browser for a one-time OAuth2 login, then caches and auto-refreshes the token in
`~/.xurl`. Credentials come from the `environment` block in `opencode.jsonc`
(`CLIENT_ID` / `CLIENT_SECRET`), which interpolate `X_CLIENT_ID` /
`X_CLIENT_SECRET` from `.env`.

The same bridge is used by the `pitch-cli` pipeline's X calls only indirectly —
the internal `src/x_api.rs` client still uses `.env` OAuth2 user tokens
(`X_USER_ACCESS_TOKEN` / `X_USER_REFRESH_TOKEN`).

---

## One-time login (browser)

```bash
# env comes from .env; register a default app once
xurl auth apps add my-app --client-id "$X_CLIENT_ID" --client-secret "$X_CLIENT_SECRET"
xurl auth oauth2 --app my-app
```

The first tool call through `xmcp` also triggers this login automatically.

### Dev app prerequisites (X Developer Portal → app → User authentication settings)

* **Callback / Redirect URI**: `http://localhost:8080/callback`
  (or set `REDIRECT_URI` to a registered alternative; the bridge default is
  `http://localhost:8080/callback`).
* **App Permissions**: `Read and write` (plus `Direct message` if DMs are needed).
* **Type of App**: `Web App, Automated App or Bot`.
* Move the app to the **Pay-per-use** package and the **Production** environment
  (`client-not-enrolled` errors otherwise).

## Headless / remote machines

No browser reachable? Authenticate out-of-band once, then the bridge reuses the
cached token:

```bash
export CLIENT_ID="$X_CLIENT_ID" CLIENT_SECRET="$X_CLIENT_SECRET"
xurl auth oauth2 --app my-app --headless   # open URL on any device, paste back the code
```

## Verify

```bash
xurl auth status     # shows registered apps + users + token state
xurl whoami          # confirms the acting account
xurl token           # prints a fresh access token (for debugging only)
```