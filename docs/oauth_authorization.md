# Automated X API v2 OAuth2 PKCE Authorization

This document describes the automated OAuth2 PKCE authorization flow in `pitch-cli` (`x-api authorize`).

---

## Overview

When X API user tokens (`X_USER_ACCESS_TOKEN` / `X_USER_REFRESH_TOKEN`) expire or get revoked, `pitch-cli x-api authorize` provides a 100% automated PKCE re-authorization flow using `agent-webbridge`.

---

## Architecture & Flow

1. **Callback Server**:
   - `pitch-cli` binds a temporary HTTP server on `http://127.0.0.1:18795`.
   - Dedicated callback URL registered in X Developer Portal:
     `http://127.0.0.1:18795/callback`

2. **PKCE Parameters**:
   - Generates SHA-256 base64url `code_verifier` and `code_challenge`.
   - Constructs authorization URL with scopes:
     `tweet.read tweet.write users.read offline.access like.read like.write`

3. **Automated Browser Approval**:
   - Sends a `navigate` request to `agent-webbridge` (`http://127.0.0.1:10086/command`) targeting the logged-in `Testing` Chrome profile.
   - Spawns a background worker that polls the accessibility snapshot for the **"Authorize app"** button.
   - Automatically issues a synthetic `click` command to `agent-webbridge` once the button is located.

4. **Token Exchange & `.env` Update**:
   - Captures the `code` parameter from X's redirect to `http://127.0.0.1:18795/callback`.
   - Sends a `POST` request to `https://api.twitter.com/2/oauth2/token` to exchange `code` + `code_verifier` for fresh tokens.
   - Automatically updates `X_USER_ACCESS_TOKEN` and `X_USER_REFRESH_TOKEN` in `.env`.

---

## Usage

Run the authorization command from `pitch-cli`:

```bash
./target/release/pitch-cli x-api authorize
```

### Custom Port / Callback URL (Optional)

```bash
./target/release/pitch-cli x-api authorize --port 18795 --redirect-uri http://127.0.0.1:18795/callback
```

---

## Developer Portal Prerequisites

In **X Developer Console → App Settings → User authentication settings**:

* **Callback URL / Redirect URI**: `http://127.0.0.1:18795/callback`
* **Website URL**: `https://trypitch.co`
* **App Permissions**: `Read and write and Direct message`
* **Type of App**: `Web App, Automated App or Bot`
