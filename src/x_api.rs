use crate::config::Config;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const API_BASE: &str = "https://api.twitter.com/2";
const TOKEN_URL: &str = "https://api.twitter.com/2/oauth2/token";
const UA: &str = "Mozilla/5.0 (pitch-x-growth-rust)";

#[derive(Debug)]
pub struct XApiError(pub String);

impl std::fmt::Display for XApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for XApiError {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MentionTweet {
    pub id: String,
    pub text: String,
    pub author_id: Option<String>,
    pub author_handle: Option<String>,
    pub created_at: Option<String>,
}

pub struct XApiClient {
    pub config: Config,
    client: reqwest::Client,
}

impl XApiClient {
    pub fn new() -> Self {
        XApiClient {
            config: Config::load(),
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.x_access)).unwrap(),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub async fn refresh_token(&mut self) -> Result<(), XApiError> {
        if self.config.x_refresh.is_empty() || self.config.x_client_id.is_empty() {
            return Err(XApiError("Missing refresh token or client ID in .env".to_string()));
        }

        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", self.config.x_refresh.as_str());
        params.insert("client_id", self.config.x_client_id.as_str());

        let mut req = self
            .client
            .post(TOKEN_URL)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, UA)
            .form(&params);

        if !self.config.x_client_secret.is_empty() {
            use base64::Engine;
            let creds = base64::engine::general_purpose::STANDARD
                .encode(format!("{}:{}", self.config.x_client_id, self.config.x_client_secret));
            req = req.header(AUTHORIZATION, format!("Basic {}", creds));
        }

        let res = req
            .send()
            .await
            .map_err(|e| XApiError(format!("Refresh network error: {}", e)))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(XApiError(format!("Refresh failed: {}", err_text)));
        }

        let data: Value = res
            .json()
            .await
            .map_err(|e| XApiError(format!("Invalid refresh JSON: {}", e)))?;

        let new_access = data["access_token"]
            .as_str()
            .ok_or_else(|| XApiError("No access_token in response".to_string()))?;
        let new_refresh = data["refresh_token"]
            .as_str()
            .unwrap_or(&self.config.x_refresh);

        let mut updates = HashMap::new();
        updates.insert("X_USER_ACCESS_TOKEN".to_string(), new_access.to_string());
        updates.insert("X_USER_REFRESH_TOKEN".to_string(), new_refresh.to_string());

        let _ = self.config.update_env(updates);
        self.config = Config::load();

        println!("[X API] OAuth2 token refreshed successfully.");
        Ok(())
    }

    pub async fn authorize(&mut self, port: u16, redirect_uri: &str) -> Result<(), XApiError> {
        if self.config.x_client_id.is_empty() {
            return Err(XApiError("Missing X_CLIENT_ID in .env".to_string()));
        }

        use base64::Engine;
        use sha2::{Digest, Sha256};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let seed = format!(
            "{:?}-{}-pitch-pkce",
            std::time::SystemTime::now(),
            std::process::id()
        );
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let hash = hasher.finalize();
        let code_verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

        let mut challenge_hasher = Sha256::new();
        challenge_hasher.update(code_verifier.as_bytes());
        let challenge_hash = challenge_hasher.finalize();
        let code_challenge =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge_hash);

        let scope = urlencoding::encode("tweet.read tweet.write users.read offline.access like.read like.write");
        let encoded_redirect = urlencoding::encode(redirect_uri);
        let auth_url = format!(
            "https://twitter.com/i/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&state=state123&code_challenge={}&code_challenge_method=S256",
            self.config.x_client_id,
            encoded_redirect,
            scope,
            code_challenge
        );

        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| XApiError(format!("Failed to bind callback listener on {}: {}", addr, e)))?;

        println!("\n========================================================");
        println!("🔐 X OAuth2 PKCE Authorization Flow");
        println!("========================================================");
        println!("Callback listener active on http://{}", addr);
        println!("Navigating Chrome to authorization page...\n");
        println!("URL: {}\n", auth_url);

        let webbridge_payload = serde_json::json!({
            "action": "navigate",
            "args": { "url": auth_url, "newTab": true },
            "profile": "Testing",
            "session": "oauth-auth"
        });
        let _ = self
            .client
            .post("http://127.0.0.1:10086/command")
            .json(&webbridge_payload)
            .send()
            .await;

        let client_clone = self.client.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            for _ in 0..10 {
                let snap_req = serde_json::json!({
                    "action": "snapshot",
                    "args": {},
                    "profile": "Testing",
                    "session": "oauth-auth"
                });
                if let Ok(res) = client_clone
                    .post("http://127.0.0.1:10086/command")
                    .json(&snap_req)
                    .send()
                    .await
                {
                    if let Ok(val) = res.json::<serde_json::Value>().await {
                        let tree = val["data"]["tree"].to_string();
                        if let Ok(re) = regex::Regex::new(r#""Authorize app".*?"ref"\s*:\s*"(@e\d+)""#) {
                            if let Some(caps) = re.captures(&tree) {
                                let btn_ref = &caps[1];
                                println!("[Auto-Click] Found 'Authorize app' button ({}), clicking automatically via agent-webbridge...", btn_ref);
                                let click_req = serde_json::json!({
                                    "action": "click",
                                    "args": { "selector": btn_ref },
                                    "profile": "Testing",
                                    "session": "oauth-auth"
                                });
                                let _ = client_clone
                                    .post("http://127.0.0.1:10086/command")
                                    .json(&click_req)
                                    .send()
                                    .await;
                                break;
                            }
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        let (mut socket, _) = listener
            .accept()
            .await
            .map_err(|e| XApiError(format!("Callback accept error: {}", e)))?;

        let mut buffer = [0u8; 2048];
        let n = socket.read(&mut buffer).await.unwrap_or(0);
        let req_str = String::from_utf8_lossy(&buffer[..n]);

        let mut code = String::new();
        if let Some(first_line) = req_str.lines().next() {
            if let Some(query_start) = first_line.find('?') {
                if let Some(query_end) = first_line[query_start..].find(' ') {
                    let query = &first_line[query_start + 1..query_start + query_end];
                    for pair in query.split('&') {
                        let mut parts = pair.split('=');
                        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                            if k == "code" {
                                code = urlencoding::decode(v).unwrap_or_default().to_string();
                            }
                        }
                    }
                }
            }
        }

        let html_response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>OAuth Authorization Successful!</h1><p>You can close this tab now and return to pitch-cli.</p></body></html>";
        let _ = socket.write_all(html_response.as_bytes()).await;
        let _ = socket.flush().await;

        if code.is_empty() {
            return Err(XApiError(
                "No authorization code found in callback request".to_string(),
            ));
        }

        println!("[✓] Captured authorization code!");
        println!("[...] Exchanging code for OAuth2 tokens...");

        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code.as_str());
        params.insert("redirect_uri", redirect_uri);
        params.insert("code_verifier", code_verifier.as_str());
        params.insert("client_id", self.config.x_client_id.as_str());

        let mut req = self
            .client
            .post(TOKEN_URL)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(USER_AGENT, UA)
            .form(&params);

        if !self.config.x_client_secret.is_empty() {
            let creds = base64::engine::general_purpose::STANDARD.encode(format!(
                "{}:{}",
                self.config.x_client_id, self.config.x_client_secret
            ));
            req = req.header(AUTHORIZATION, format!("Basic {}", creds));
        }

        let res = req
            .send()
            .await
            .map_err(|e| XApiError(format!("Token exchange network error: {}", e)))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(XApiError(format!("Token exchange failed: {}", err_text)));
        }

        let data: Value = res
            .json()
            .await
            .map_err(|e| XApiError(format!("Invalid token JSON response: {}", e)))?;

        let new_access = data["access_token"]
            .as_str()
            .ok_or_else(|| XApiError("No access_token in token response".to_string()))?;
        let new_refresh = data["refresh_token"]
            .as_str()
            .ok_or_else(|| XApiError("No refresh_token in token response".to_string()))?;

        let mut updates = HashMap::new();
        updates.insert("X_USER_ACCESS_TOKEN".to_string(), new_access.to_string());
        updates.insert("X_USER_REFRESH_TOKEN".to_string(), new_refresh.to_string());

        let _ = self.config.update_env(updates);
        self.config = Config::load();

        println!("[✓] OAuth2 tokens successfully acquired and saved to .env!");
        Ok(())
    }

    pub async fn request<T: serde::de::DeserializeOwned>(
        &mut self,
        method: reqwest::Method,
        path: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<Value>,
    ) -> Result<T, XApiError> {
        let mut retried = false;

        loop {
            let mut url = format!("{}{}", API_BASE, path);
            if let Some(q) = query {
                let query_str = q
                    .iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&");
                url = format!("{}?{}", url, query_str);
            }

            let mut req = self.client.request(method.clone(), &url).headers(self.headers());

            if let Some(b) = body.clone() {
                req = req.json(&b);
            }

            let res = req
                .send()
                .await
                .map_err(|e| XApiError(format!("Network error: {}", e)))?;

            if res.status() == reqwest::StatusCode::UNAUTHORIZED && !retried {
                println!("[X API 401] Attempting auto token refresh...");
                retried = true;
                if self.refresh_token().await.is_ok() {
                    continue;
                }
            }

            if !res.status().is_success() {
                let err_text = res.text().await.unwrap_or_default();
                return Err(XApiError(format!("API Error ({}): {}", url, err_text)));
            }

            return res
                .json::<T>()
                .await
                .map_err(|e| XApiError(format!("JSON parse error: {}", e)));
        }
    }

    pub async fn get_me(&mut self) -> Result<Value, XApiError> {
        let res: Value = self
            .request(
                reqwest::Method::GET,
                "/users/me",
                Some(&[("user.fields", "id,name,username")]),
                None,
            )
            .await?;
        Ok(res["data"].clone())
    }

    pub async fn lookup_user(&mut self, username: &str) -> Result<Value, XApiError> {
        let clean = username.trim_start_matches('@');
        let path = format!("/users/by/username/{}", clean);
        let res: Value = self
            .request(reqwest::Method::GET, &path, None, None)
            .await?;
        Ok(res["data"].clone())
    }

    pub async fn get_user_id(&mut self) -> Result<String, XApiError> {
        if !self.config.x_user_id.is_empty() {
            return Ok(self.config.x_user_id.clone());
        }
        let me = self.get_me().await?;
        if let Some(id) = me["id"].as_str() {
            let mut updates = HashMap::new();
            updates.insert("X_USER_ID".to_string(), id.to_string());
            let _ = self.config.update_env(updates);
            return Ok(id.to_string());
        }
        Err(XApiError("Could not resolve X User ID".to_string()))
    }

    pub async fn get_mentions(
        &mut self,
        since_id: Option<&str>,
        max_results: usize,
    ) -> Result<Vec<MentionTweet>, XApiError> {
        let user_id = self.get_user_id().await?;
        let path = format!("/users/{}/mentions", user_id);

        let max_str = max_results.min(20).to_string();
        let mut query_params = vec![
            ("max_results", max_str.as_str()),
            ("expansions", "author_id,referenced_tweets.id"),
            ("tweet.fields", "created_at,text,author_id,id"),
            ("user.fields", "id,name,username"),
        ];

        if let Some(sid) = since_id {
            query_params.push(("since_id", sid));
        }

        let res: Value = self
            .request(
                reqwest::Method::GET,
                &path,
                Some(&query_params),
                None,
            )
            .await?;

        let mut users_map = HashMap::new();
        if let Some(users) = res["includes"]["users"].as_array() {
            for u in users {
                if let (Some(id), Some(uname)) = (u["id"].as_str(), u["username"].as_str()) {
                    users_map.insert(id.to_string(), format!("@{}", uname));
                }
            }
        }

        let mut mentions = Vec::new();
        if let Some(tweets) = res["data"].as_array() {
            for t in tweets {
                let id = t["id"].as_str().unwrap_or_default().to_string();
                let text = t["text"].as_str().unwrap_or_default().to_string();
                let author_id = t["author_id"].as_str().map(|s| s.to_string());
                let author_handle = author_id.as_ref().and_then(|aid| users_map.get(aid).cloned());
                let created_at = t["created_at"].as_str().map(|s| s.to_string());

                mentions.push(MentionTweet {
                    id,
                    text,
                    author_id,
                    author_handle,
                    created_at,
                });
            }
        }

        Ok(mentions)
    }

    pub async fn post_tweet(
        &mut self,
        text: &str,
        reply_to_tweet_id: Option<&str>,
    ) -> Result<String, XApiError> {
        let mut body = serde_json::json!({ "text": text });
        if let Some(reply_id) = reply_to_tweet_id {
            body["reply"] = serde_json::json!({ "in_reply_to_tweet_id": reply_id });
        }

        let res: Value = self
            .request(reqwest::Method::POST, "/tweets", None, Some(body))
            .await?;

        let id = res["data"]["id"]
            .as_str()
            .ok_or_else(|| XApiError("Tweet creation response missing ID".to_string()))?;
        Ok(id.to_string())
    }

    pub async fn like_tweet(&mut self, tweet_id: &str) -> Result<bool, XApiError> {
        let user_id = self.get_user_id().await?;
        let path = format!("/users/{}/likes", user_id);
        let body = serde_json::json!({ "tweet_id": tweet_id });

        let res: Value = self
            .request(reqwest::Method::POST, &path, None, Some(body))
            .await?;

        Ok(res["data"]["liked"].as_bool().unwrap_or(false))
    }

    pub async fn search_recent(
        &mut self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<Value>, XApiError> {
        let max_str = max_results.min(20).to_string();
        let query_params = [
            ("query", query),
            ("max_results", max_str.as_str()),
            ("expansions", "author_id"),
            ("tweet.fields", "created_at,text,author_id"),
            ("user.fields", "id,username,name"),
        ];

        let res: Value = self
            .request(
                reqwest::Method::GET,
                "/tweets/search/recent",
                Some(&query_params),
                None,
            )
            .await?;

        let mut users_map = HashMap::new();
        if let Some(users) = res["includes"]["users"].as_array() {
            for u in users {
                if let (Some(id), Some(uname)) = (u["id"].as_str(), u["username"].as_str()) {
                    users_map.insert(id.to_string(), format!("@{}", uname));
                }
            }
        }

        let mut results = Vec::new();
        if let Some(tweets) = res["data"].as_array() {
            for t in tweets {
                let id = t["id"].as_str().unwrap_or_default();
                let text = t["text"].as_str().unwrap_or_default();
                let author_id = t["author_id"].as_str().unwrap_or_default();
                let handle = users_map.get(author_id).cloned().unwrap_or_default();

                results.push(serde_json::json!({
                    "id": id,
                    "author": handle,
                    "created_at": t["created_at"],
                    "text": text
                }));
            }
        }

        Ok(results)
    }
}
