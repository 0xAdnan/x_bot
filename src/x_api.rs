use crate::config::Config;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
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
