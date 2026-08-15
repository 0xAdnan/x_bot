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
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    fn oauth1_header(&self, method: &str, path: &str, query: Option<&[(&str, &str)]>) -> String {
        let api_key = std::env::var("X_API_KEY").unwrap_or_default();
        let api_secret = std::env::var("X_API_SECRET").unwrap_or_default();
        let user_token = std::env::var("X_USER_ACCESS_TOKEN").unwrap_or_default();
        let user_secret = std::env::var("X_USER_ACCESS_SECRET").unwrap_or_default();

        let api_key = crate::config::try_b64decode(&api_key);
        let api_secret = crate::config::try_b64decode(&api_secret);
        let user_token = crate::config::try_b64decode(&user_token);
        let user_secret = crate::config::try_b64decode(&user_secret);

        let url_str = format!("{}{}", API_BASE, path);
        let nonce = format!("{:x}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(123456789));
        let timestamp = chrono::Utc::now().timestamp().to_string();

        let mut all_params = std::collections::BTreeMap::new();
        all_params.insert("oauth_consumer_key", api_key.as_str());
        all_params.insert("oauth_nonce", nonce.as_str());
        all_params.insert("oauth_signature_method", "HMAC-SHA1");
        all_params.insert("oauth_timestamp", timestamp.as_str());
        all_params.insert("oauth_token", user_token.as_str());
        all_params.insert("oauth_version", "1.0");

        if let Some(qp) = query {
            for (k, v) in qp {
                all_params.insert(*k, *v);
            }
        }

        let sorted_params = all_params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let base_string = format!(
            "{}&{}&{}",
            method,
            urlencoding::encode(&url_str),
            urlencoding::encode(&sorted_params)
        );

        let signing_key = format!(
            "{}&{}",
            urlencoding::encode(&api_secret),
            urlencoding::encode(&user_secret)
        );

        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        type HmacSha1 = Hmac<Sha1>;

        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes()).unwrap();
        mac.update(base_string.as_bytes());
        let result = mac.finalize();
        use base64::Engine;
        let signature = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());

        format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_nonce=\"{}\", oauth_signature=\"{}\", oauth_signature_method=\"HMAC-SHA1\", oauth_timestamp=\"{}\", oauth_token=\"{}\", oauth_version=\"1.0\"",
            urlencoding::encode(&api_key),
            urlencoding::encode(&nonce),
            urlencoding::encode(&signature),
            urlencoding::encode(&timestamp),
            urlencoding::encode(&user_token)
        )
    }

    pub async fn refresh_token(&mut self) -> Result<(), XApiError> {
        if self.config.x_refresh.is_empty() || self.config.x_client_id.is_empty() {
            return Err(XApiError("Missing refresh token or client ID in .env".to_string()));
        }

        // Try candidate client IDs: raw vs decoded
        let mut candidates = vec![self.config.x_client_id.clone()];
        if let Ok(raw_env_cid) = std::env::var("X_CLIENT_ID") {
            if raw_env_cid != self.config.x_client_id {
                candidates.push(raw_env_cid);
            }
        }

        let mut success_data: Option<Value> = None;

        for cid in candidates {
            let mut params = HashMap::new();
            params.insert("grant_type", "refresh_token");
            params.insert("refresh_token", self.config.x_refresh.as_str());
            params.insert("client_id", cid.as_str());

            // Try Public Mode
            let res1 = self
                .client
                .post(TOKEN_URL)
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(USER_AGENT, UA)
                .form(&params)
                .send()
                .await;

            if let Ok(resp) = res1 {
                let status = resp.status();
                let body_text = resp.text().await.unwrap_or_default();
                println!("[X API Refresh Mode ({})] Body: {}", status, body_text);
                if status.is_success() {
                    if let Ok(data) = serde_json::from_str::<Value>(&body_text) {
                        success_data = Some(data);
                        break;
                    }
                }
            }
        }

        // Try 2: Confidential Client mode with Basic Auth header
        if success_data.is_none() && !self.config.x_client_secret.is_empty() {
            let mut params = HashMap::new();
            params.insert("grant_type", "refresh_token");
            params.insert("refresh_token", self.config.x_refresh.as_str());
            params.insert("client_id", self.config.x_client_id.as_str());

            use base64::Engine;
            let creds = base64::engine::general_purpose::STANDARD.encode(format!(
                "{}:{}",
                urlencoding::encode(&self.config.x_client_id),
                urlencoding::encode(&self.config.x_client_secret)
            ));

            let res2 = self
                .client
                .post(TOKEN_URL)
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(AUTHORIZATION, format!("Basic {}", creds))
                .header(USER_AGENT, UA)
                .form(&params)
                .send()
                .await;

            if let Ok(resp) = res2 {
                let status = resp.status();
                let body_text = resp.text().await.unwrap_or_default();
                println!("[X API Refresh Confidential Mode ({})] Body: {}", status, body_text);
                if status.is_success() {
                    if let Ok(data) = serde_json::from_str::<Value>(&body_text) {
                        success_data = Some(data);
                    }
                }
            }
        }

        let data = success_data.ok_or_else(|| {
            XApiError(
                "Refresh token exchange failed on both Public and Confidential client modes."
                    .to_string(),
            )
        })?;

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

            let auth_header = self.oauth1_header(method.as_str(), path, query);
            let mut req = self
                .client
                .request(method.clone(), &url)
                .headers(self.headers())
                .header(AUTHORIZATION, auth_header);

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

        match self
            .request::<Value>(reqwest::Method::POST, "/tweets", None, Some(body))
            .await
        {
            Ok(res) => {
                if let Some(id) = res["data"]["id"].as_str() {
                    return Ok(id.to_string());
                }
                Err(XApiError("Tweet creation response missing ID".to_string()))
            }
            Err(e) => {
                println!(
                    "[X API Primary Failed]: {}. Executing Playwright browser fallback...",
                    e
                );
                let state_file = "/home/adnan/x_bot/.browser-profile/storageState_trypitchdotco.json";
                let target_url = match reply_to_tweet_id {
                    Some(tid) => format!("https://x.com/i/status/{}", tid),
                    None => "https://x.com/compose/post".to_string(),
                };
                let py_script = format!(
                    "import asyncio, re, urllib.request, os\n\
                    from playwright.async_api import async_playwright\n\
                    async def run():\n\
                    \tasync with async_playwright() as p:\n\
                    \t\tbrowser = await p.chromium.launch(headless=True)\n\
                    \t\tcontext = await browser.new_context(storage_state='{}')\n\
                    \t\tpage = await context.new_page()\n\
                    \t\tawait page.goto('{}', wait_until='domcontentloaded')\n\
                    \t\tawait page.wait_for_timeout(3000)\n\
                    \t\treply_icon = page.locator('button[data-testid=\"reply\"]').first\n\
                    \t\tif await reply_icon.count() > 0:\n\
                    \t\t\tawait reply_icon.click()\n\
                    \t\t\tawait page.wait_for_timeout(1000)\n\
                    \t\ttext_val = {:?}\n\
                    \t\tm = re.search(r'https://[^\\s]+\\.mp4', text_val)\n\
                    \t\tif m:\n\
                    \t\t\ttry:\n\
                    \t\t\t\tvideo_url = m.group(0)\n\
                    \t\t\t\treq = urllib.request.Request(video_url, headers={{'User-Agent': 'Mozilla/5.0'}})\n\
                    \t\t\t\twith urllib.request.urlopen(req) as resp, open('/tmp/upload_video.mp4', 'wb') as f:\n\
                    \t\t\t\t\tf.write(resp.read())\n\
                    \t\t\t\tfile_inp = page.locator('input[data-testid=\"fileInput\"]').first\n\
                    \t\t\t\tif await file_inp.count() > 0:\n\
                    \t\t\t\t\tawait file_inp.set_input_files('/tmp/upload_video.mp4')\n\
                    \t\t\t\t\tawait page.wait_for_timeout(12000)\n\
                    \t\t\texcept Exception as ve: print('[Video Attach Warning]:', ve)\n\
                    \t\tbox = page.locator('div[data-testid=\"tweetTextarea_0\"]').first\n\
                    \t\tif await box.count() > 0:\n\
                    \t\t\tawait box.click()\n\
                    \t\t\tawait box.fill(text_val)\n\
                    \t\t\tawait page.wait_for_timeout(1000)\n\
                    \t\t\tbtn = page.locator('button[data-testid=\"tweetButton\"], button[data-testid=\"tweetButtonInline\"]').first\n\
                    \t\t\tif await btn.count() > 0:\n\
                    \t\t\t\tawait btn.click()\n\
                    \t\t\t\tawait page.wait_for_timeout(5000)\n\
                    \t\t\t\tprint('[Browser Fallback Success] Native Video Tweet posted via Playwright')\n\
                    \t\tif os.path.exists('/tmp/upload_video.mp4'):\n\
                    \t\t\ttry: os.remove('/tmp/upload_video.mp4')\n\
                    \t\t\texcept: pass\n\
                    \t\tawait browser.close()\n\
                    asyncio.run(run())",
                    state_file, target_url, text
                );
                let _ = std::process::Command::new("python3")
                    .arg("-c")
                    .arg(py_script)
                    .output();
                Ok("browser_fallback_posted".to_string())
            }
        }
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
        println!("[Silent Browser Read]: Searching X for '{}' via Playwright Chromium (@adnanspitch)...", query);
        let state_file = "/home/adnan/x_bot/.browser-profile/storageState.json";
        let search_url = format!("https://x.com/search?q={}&f=live", urlencoding::encode(query));

        let py_script = format!(
            "import asyncio, json, re\n\
            from playwright.async_api import async_playwright\n\
            async def run():\n\
            \tasync with async_playwright() as p:\n\
            \t\tbrowser = await p.chromium.launch(headless=True)\n\
            \t\tcontext = await browser.new_context(storage_state='{}')\n\
            \t\tpage = await context.new_page()\n\
            \t\tawait page.goto('{}', wait_until='domcontentloaded')\n\
            \t\tawait page.wait_for_timeout(4000)\n\
            \t\ttweets = await page.locator('article[data-testid=\"tweet\"]').all()\n\
            \t\tres = []\n\
            \t\tfor t in tweets[:{}]:\n\
            \t\t\ttry:\n\
            \t\t\t\ttext = await t.inner_text()\n\
            \t\t\t\tclean = text.replace('\\n', ' ')\n\
            \t\t\t\thandle_m = re.search(r'@(\\w+)', clean)\n\
            \t\t\t\thandle = '@' + handle_m.group(1) if handle_m else '@prospect'\n\
            \t\t\t\tres.append({{'id': 'browser_tweet', 'author': handle, 'text': clean}})\n\
            \t\t\texcept Exception: pass\n\
            \t\tprint(json.dumps(res))\n\
            \t\tawait browser.close()\n\
            asyncio.run(run())",
            state_file, search_url, max_results
        );

        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg(py_script)
            .output();

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Ok(parsed) = serde_json::from_str::<Vec<Value>>(&text) {
                return Ok(parsed);
            }
        }

        Ok(Vec::new())
    }
}
