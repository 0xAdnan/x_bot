use crate::{
    db::{Database, MentionJob},
    pitch_mcp::create_demo_video,
    x_api::XApiClient,
};
use regex::Regex;

fn extract_url(text: &str) -> String {
    let re_http = Regex::new(r"https?://[^\s]+").unwrap();
    let re_domain = Regex::new(r"([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?)").unwrap();

    if let Some(m) = re_http.find(text) {
        return m.as_str().to_string();
    }
    if let Some(m) = re_domain.find(text) {
        let domain = m.as_str();
        return format!("https://{}", domain);
    }
    "N/A".to_string()
}

pub async fn process_mention_inbox(dry_run: bool, no_ack: bool) -> Result<(usize, usize), String> {
    println!("=== [TRIGGERED MENTION INGESTION (WEBHOOK PASS)] ===");

    let db = Database::open().map_err(|e| format!("DB Error: {}", e))?;
    let mut x_client = XApiClient::new();

    let mentions = x_client
        .get_mentions(None, 20)
        .await
        .map_err(|e| format!("Failed to fetch mentions: {}", e))?;

    println!("[Inbox] Fetched {} recent mentions.", mentions.len());

    let mut new_count = 0;
    let mut jobs_count = 0;

    for tweet in mentions {
        let tweet_id = tweet.id.clone();
        let author_handle = tweet
            .author_handle
            .clone()
            .unwrap_or_else(|| "@user".to_string());
        let text = tweet.text.clone();

        if author_handle.eq_ignore_ascii_case("@trypitchdotco") {
            continue;
        }

        if let Ok(Some(_)) = db.get_mention_job_by_tweet_id(&tweet_id) {
            continue;
        }

        new_count += 1;
        let target_url = extract_url(&text);

        println!("\n--------------------------------------------------");
        println!("[NEW MENTION DETECTED] Tweet ID: {}", tweet_id);
        println!("User: {}", author_handle);
        println!("Text: {}", text.replace('\n', " "));
        println!("Target URL: {}", target_url);

        if target_url == "N/A" || target_url.contains("s3.trypitch.co") {
            println!("[Inbox] No external product URL found in mention. Recording status: no_url_found");
            let _ = db.upsert_mention_job(&MentionJob {
                id: None,
                tweet_id: tweet_id.clone(),
                user_handle: author_handle.clone(),
                target_url: "N/A".to_string(),
                editor_job_id: None,
                status: "no_url_found".to_string(),
                s3_video_url: None,
                x_reply_id: None,
                created_at: None,
                updated_at: None,
            });
            continue;
        }

        if !no_ack && !dry_run {
            let ack_text = format!(
                "Cool {}, we're on it! 🚀 Generating your cinematic demo for {} now, we'll get back to you with the video link right here soon!",
                author_handle, target_url
            );
            match x_client.post_tweet(&ack_text, Some(&tweet_id)).await {
                Ok(reply_id) => println!("[X Receipt Reply Sent] Reply Tweet ID: {}", reply_id),
                Err(e) => println!("[Ack Warning] Could not send receipt reply: {}", e),
            }
        }

        if dry_run {
            println!("[DRY RUN] Would trigger Pitch MCP for {}", target_url);
            continue;
        }

        println!("Triggering Pitch MCP video creation for {}...", target_url);
        match create_demo_video(&target_url, None).await {
            Ok(res) => {
                let job_id = res["jobId"].as_str().unwrap_or_default().to_string();
                if !job_id.is_empty() {
                    println!("[Pitch MCP Success] Job ID: {}", job_id);
                    let _ = db.upsert_mention_job(&MentionJob {
                        id: None,
                        tweet_id: tweet_id.clone(),
                        user_handle: author_handle.clone(),
                        target_url: target_url.clone(),
                        editor_job_id: Some(job_id.clone()),
                        status: "rendering".to_string(),
                        s3_video_url: None,
                        x_reply_id: None,
                        created_at: None,
                        updated_at: None,
                    });
                    jobs_count += 1;
                } else {
                    let _ = db.upsert_mention_job(&MentionJob {
                        id: None,
                        tweet_id: tweet_id.clone(),
                        user_handle: author_handle.clone(),
                        target_url: target_url.clone(),
                        editor_job_id: None,
                        status: "submitted".to_string(),
                        s3_video_url: None,
                        x_reply_id: None,
                        created_at: None,
                        updated_at: None,
                    });
                }
            }
            Err(e) => println!("[Pitch MCP Error]: {}", e),
        }
    }

    println!(
        "\n=== [INBOX COMPLETE] New Mentions: {}, Jobs Triggered: {} ===",
        new_count, jobs_count
    );
    Ok((new_count, jobs_count))
}
