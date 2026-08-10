use crate::config::Config;
use serde_json::Value;

const MCP_URL: &str = "https://api.trypitch.co/mcp";
const UA: &str = "Mozilla/5.0 (pitch-x-growth-rust)";

pub async fn call_mcp_tool(tool_name: &str, args: Value) -> Result<Value, String> {
    let cfg = Config::load();
    if cfg.pitch_api_key.is_empty() {
        return Err("Missing PITCH_API_KEY in .env configuration".to_string());
    }

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": chrono::Utc::now().timestamp_millis(),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": args
        }
    });

    let client = reqwest::Client::new();
    let res = client
        .post(MCP_URL)
        .header("Authorization", format!("Bearer {}", cfg.pitch_api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("User-Agent", UA)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Pitch MCP Network Error: {}", e))?;

    let status = res.status();
    if !status.is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Pitch MCP Error ({}): {}", status, err_text));
    }

    let text = res.text().await.map_err(|e| e.to_string())?;

    for line in text.lines() {
        if line.starts_with("data: ") {
            if let Ok(parsed) = serde_json::from_str::<Value>(&line[6..]) {
                if let Some(content) = parsed["result"]["content"].as_array() {
                    if !content.is_empty() && content[0]["type"] == "text" {
                        if let Some(txt) = content[0]["text"].as_str() {
                            if let Ok(json_obj) = serde_json::from_str::<Value>(txt) {
                                return Ok(json_obj);
                            }
                        }
                    }
                }
            }
        }
    }

    Err("Pitch MCP response contained no valid SSE json data".to_string())
}

pub async fn create_demo_video(
    url: &str,
    custom_instructions: Option<&str>,
) -> Result<Value, String> {
    let instructions = custom_instructions.unwrap_or_else(|| {
        "Create a cinematic, polished product demo of this product. Highlight key features, value proposition, and user experience."
    });

    let args = serde_json::json!({
        "url": url,
        "instructions": instructions,
        "voice": "Charon",
        "subtitles": false,
        "theme": "light",
        "background": "ocean",
        "shape": "rounded",
        "inset": "0.75",
        "browserHeader": "light"
    });

    call_mcp_tool("create_demo_video", args).await
}

pub async fn get_job_status(job_id: &str) -> Result<Value, String> {
    call_mcp_tool("get_job", serde_json::json!({ "jobId": job_id })).await
}

pub async fn get_credits() -> Result<Value, String> {
    call_mcp_tool("get_credits", serde_json::json!({})).await
}

pub fn extract_s3_url(status_result: &Value) -> String {
    let artifacts = &status_result["artifacts"];
    if let Some(url) = artifacts["final_with_cards"].as_str() {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    if let Some(url) = artifacts["video"].as_str() {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    if let Some(url) = artifacts["mp4"].as_str() {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    if let Some(url) = status_result["s3_url"].as_str() {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    if let Some(url) = status_result["video"].as_str() {
        if !url.is_empty() {
            return url.to_string();
        }
    }
    String::new()
}
