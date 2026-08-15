use crate::{
    db::{Database, MentionJob},
    pitch_mcp::{create_demo_video, create_launch_video},
    x_api::XApiClient,
};
use regex::Regex;

fn extract_url(text: &str) -> String {
    let re_http = Regex::new(r"https?://\s*([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?)").unwrap();
    let re_domain = Regex::new(r"([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:/[^\s]*)?)").unwrap();

    if let Some(caps) = re_http.captures(text) {
        if let Some(m) = caps.get(1) {
            return format!("https://{}", m.as_str());
        }
    }
    if let Some(m) = re_domain.find(text) {
        let domain = m.as_str();
        return format!("https://{}", domain);
    }
    "N/A".to_string()
}

#[derive(Debug, PartialEq, Eq)]
pub enum MentionIntent {
    LaunchVideo(String),         // Explicit launch video request + valid product URL
    DemoVideo(String),           // Explicit demo / walkthrough request + valid product URL
    ConversationWithUrl(String), // Mention contains a link, but user is NOT asking for a video
    Conversation,                // General chat, praise, question, or discussion
}

pub fn classify_mention_intent(text: &str) -> MentionIntent {
    let lower = text.to_lowercase();
    let url = extract_url(text);

    let is_valid_product_url = url != "N/A"
        && !url.contains("trypitch.co")
        && !url.contains("twitter.com")
        && !url.contains("x.com")
        && !url.contains("t.co/")
        && !url.contains("localhost")
        && !url.contains("loca.lt")
        && !url.contains("vercel.app");

    if !is_valid_product_url {
        return MentionIntent::Conversation;
    }

    // 1. Explicit Launch Video intent
    let launch_triggers = [
        "launch video", "launch demo", "product hunt launch", "for launch",
        "make a launch", "create a launch", "generate a launch",
        "build a launch", "launch walkthrough", "launch clip", "launching"
    ];
    if launch_triggers.iter().any(|t| lower.contains(t)) {
        return MentionIntent::LaunchVideo(url);
    }

    // 2. Explicit Demo Video intent
    let demo_triggers = [
        "make a demo", "create a demo", "generate a demo", "record a demo",
        "make a video", "create a video", "generate a video", "record a video",
        "show me a demo", "build a demo", "demo for", "demo of",
        "walkthrough for", "walkthrough of", "can you make", "can you create",
        "can you demo", "can you record", "make me a", "generate me a",
        "cook a demo", "cook a video", "video for", "demo this",
        "video demo", "make demo", "generate demo", "give me a demo", "give me a video"
    ];
    if demo_triggers.iter().any(|t| lower.contains(t)) {
        return MentionIntent::DemoVideo(url);
    }

    // 3. User included a URL, but did not ask to make a video
    MentionIntent::ConversationWithUrl(url)
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
        let intent = classify_mention_intent(&text);

        println!("\n--------------------------------------------------");
        println!("[NEW MENTION DETECTED] Tweet ID: {}", tweet_id);
        println!("User: {}", author_handle);
        println!("Text: {}", text.replace('\n', " "));
        println!("Classified Intent: {:?}", intent);

        match intent {
            MentionIntent::LaunchVideo(target_url) => {
                let clean_user = author_handle.replace('@', "").to_lowercase();
                let clean_domain = target_url.replace("https://", "").replace("http://", "").replace("www.", "").split('/').next().unwrap_or("your app").to_string();

                let previous_user_jobs = db.count_jobs_by_user(&author_handle).unwrap_or(0);
                if previous_user_jobs >= 1 {
                    println!("[Inbox FOLLOWER GATE] User {} has {} previous jobs. Enforcing follow gate...", author_handle, previous_user_jobs);
                    let follow_gate_msg = format!(
                        "glad you liked the first one @{}. we just ask for a quick follow on @trypitchdotco for extra free demos and launch videos, drop your link again right after and we got you",
                        clean_user
                    );
                    if !dry_run {
                        match x_client.post_tweet(&follow_gate_msg, Some(&tweet_id)).await {
                            Ok(rid) => println!("[Follower Gate Reply Sent] Reply Tweet ID: {}", rid),
                            Err(e) => println!("[Follower Gate Reply Warning]: {}", e),
                        }
                    }
                    let _ = db.upsert_mention_job(&MentionJob {
                        id: None,
                        tweet_id: tweet_id.clone(),
                        user_handle: author_handle.clone(),
                        target_url: target_url.clone(),
                        editor_job_id: None,
                        status: "follow_required".to_string(),
                        s3_video_url: None,
                        x_reply_id: None,
                        tweet_text: Some(text.clone()),
                        created_at: None,
                        updated_at: None,
                    });
                    continue;
                }

                if !no_ack && !dry_run {
                    let ack_text = format!(
                        "on it @{}. cooking up a launch video for {} right now, will drop the video right here when it's done",
                        clean_user, clean_domain
                    );
                    match x_client.post_tweet(&ack_text, Some(&tweet_id)).await {
                        Ok(reply_id) => println!("[X Receipt Reply Sent] Reply Tweet ID: {}", reply_id),
                        Err(e) => println!("[Ack Warning] Could not send receipt reply: {}", e),
                    }
                }

                if dry_run {
                    println!("[DRY RUN] Would trigger Pitch MCP create_launch_video for {}", target_url);
                    continue;
                }

                println!("Triggering Pitch MCP create_launch_video for {}...", target_url);
                let project_name = format!("{}-launch-{}", clean_domain, tweet_id.replace('-', "_"));
                let prompt_text = format!("Create a cinematic 45-second product launch video for {}. Target startup founders and product marketers. Emphasize AI-generated product demos, polished motion graphics, and fast sharing. Use an energetic premium style with concise narration.", target_url);
                match create_launch_video(&project_name, &prompt_text, Some("optional-library-track.mp3")).await {
                    Ok(res) => {
                        let jid = res["jobId"].as_str().unwrap_or_default().to_string();
                        let final_job_id = if !jid.is_empty() { jid } else { format!("launch:{}", project_name) };
                        println!("[Pitch MCP Success] Launch Job ID: {}", final_job_id);
                        let _ = db.upsert_mention_job(&MentionJob {
                            id: None,
                            tweet_id: tweet_id.clone(),
                            user_handle: author_handle.clone(),
                            target_url: target_url.clone(),
                            editor_job_id: Some(final_job_id),
                            status: "rendering".to_string(),
                            s3_video_url: None,
                            x_reply_id: None,
                            tweet_text: Some(text.clone()),
                            created_at: None,
                            updated_at: None,
                        });
                        jobs_count += 1;
                    }
                    Err(e) => println!("[Pitch MCP Error]: {}", e),
                }
            }

            MentionIntent::DemoVideo(target_url) => {
                let clean_user = author_handle.replace('@', "").to_lowercase();
                let clean_domain = target_url.replace("https://", "").replace("http://", "").replace("www.", "").split('/').next().unwrap_or("your app").to_string();

                let previous_user_jobs = db.count_jobs_by_user(&author_handle).unwrap_or(0);
                if previous_user_jobs >= 1 {
                    println!("[Inbox FOLLOWER GATE] User {} has {} previous jobs. Enforcing follow gate...", author_handle, previous_user_jobs);
                    let follow_gate_msg = format!(
                        "glad you liked the first one @{}. we just ask for a quick follow on @trypitchdotco for extra free demos and launch videos, drop your link again right after and we got you",
                        clean_user
                    );
                    if !dry_run {
                        match x_client.post_tweet(&follow_gate_msg, Some(&tweet_id)).await {
                            Ok(rid) => println!("[Follower Gate Reply Sent] Reply Tweet ID: {}", rid),
                            Err(e) => println!("[Follower Gate Reply Warning]: {}", e),
                        }
                    }
                    let _ = db.upsert_mention_job(&MentionJob {
                        id: None,
                        tweet_id: tweet_id.clone(),
                        user_handle: author_handle.clone(),
                        target_url: target_url.clone(),
                        editor_job_id: None,
                        status: "follow_required".to_string(),
                        s3_video_url: None,
                        x_reply_id: None,
                        tweet_text: Some(text.clone()),
                        created_at: None,
                        updated_at: None,
                    });
                    continue;
                }

                if !no_ack && !dry_run {
                    let ack_text = format!(
                        "on it @{}. generating a walkthrough for {} now, will post the video link here in a minute",
                        clean_user, clean_domain
                    );
                    match x_client.post_tweet(&ack_text, Some(&tweet_id)).await {
                        Ok(reply_id) => println!("[X Receipt Reply Sent] Reply Tweet ID: {}", reply_id),
                        Err(e) => println!("[Ack Warning] Could not send receipt reply: {}", e),
                    }
                }

                if dry_run {
                    println!("[DRY RUN] Would trigger Pitch MCP create_demo_video for {}", target_url);
                    continue;
                }

                println!("Triggering Pitch MCP create_demo_video for {}...", target_url);
                match create_demo_video(&target_url, None, None, None, None, None).await {
                    Ok(res) => {
                        let job_id = res["jobId"].as_str().unwrap_or_default().to_string();
                        if !job_id.is_empty() {
                            println!("[Pitch MCP Success] Demo Job ID: {}", job_id);
                            let _ = db.upsert_mention_job(&MentionJob {
                                id: None,
                                tweet_id: tweet_id.clone(),
                                user_handle: author_handle.clone(),
                                target_url: target_url.clone(),
                                editor_job_id: Some(job_id),
                                status: "rendering".to_string(),
                                s3_video_url: None,
                                x_reply_id: None,
                                tweet_text: Some(text.clone()),
                                created_at: None,
                                updated_at: None,
                            });
                            jobs_count += 1;
                        }
                    }
                    Err(e) => println!("[Pitch MCP Error]: {}", e),
                }
            }

            MentionIntent::ConversationWithUrl(target_url) => {
                println!("[INTENT: CONVERSATION WITH URL] User: {} | URL: {} (No video requested - DO NOT call Pitch MCP)", author_handle, target_url);
                let clean_user = author_handle.replace('@', "").to_lowercase();
                let clean_domain = target_url.replace("https://", "").replace("http://", "").replace("www.", "").split('/').next().unwrap_or("your app").to_string();
                let reply_msg = format!(
                    "checked out {} @{}, looks super clean. if you ever need a quick 60s video walkthrough or launch demo for it, just tag @trypitchdotco anytime and we got you",
                    clean_domain, clean_user
                );
                if !dry_run {
                    match x_client.post_tweet(&reply_msg, Some(&tweet_id)).await {
                        Ok(rid) => println!("[Conversation Reply Sent] Reply Tweet ID: {}", rid),
                        Err(e) => println!("[Conversation Reply Warning]: {}", e),
                    }
                }
                let _ = db.upsert_mention_job(&MentionJob {
                    id: None,
                    tweet_id: tweet_id.clone(),
                    user_handle: author_handle.clone(),
                    target_url: target_url.clone(),
                    editor_job_id: None,
                    status: "conversation".to_string(),
                    s3_video_url: None,
                    x_reply_id: None,
                    tweet_text: Some(text.clone()),
                    created_at: None,
                    updated_at: None,
                });
            }

            MentionIntent::Conversation => {
                println!("[INTENT: CONVERSATIONAL CHAT] User: {} (No video requested - DO NOT call Pitch MCP)", author_handle);
                let lower_text = text.to_lowercase();
                let clean_user = author_handle.replace('@', "").to_lowercase();

                let reply_msg = if lower_text.contains("connect") || lower_text.contains("dm") || lower_text.contains("chat") {
                    format!("sounds good @{}, shoot a dm anytime or drop your project link here if u want us to check it out", clean_user)
                } else if lower_text.contains("thank") || lower_text.contains("nice") || lower_text.contains("cool") || lower_text.contains("awesome") || lower_text.contains("fire") || lower_text.contains("love") {
                    format!("appreciate the love @{}. let us know anytime if you want a quick walkthrough for anything you're shipping", clean_user)
                } else if lower_text.contains("how") || lower_text.contains("what") || lower_text.contains("stack") || lower_text.contains("tech") {
                    format!("hey @{}, we turn written walkthroughs into 1080p narrated video demos using automated browser recording + voiceover. just mention @trypitchdotco make a demo for example.com to try it out", clean_user)
                } else {
                    format!("hey @{}, good to see you on the timeline. whenever you need a demo or launch video, just mention @trypitchdotco make a demo for example.com and we got you", clean_user)
                };

                if !dry_run {
                    match x_client.post_tweet(&reply_msg, Some(&tweet_id)).await {
                        Ok(rid) => println!("[Conversational Reply Sent] Reply Tweet ID: {}", rid),
                        Err(e) => println!("[Conversational Reply Warning]: {}", e),
                    }
                }
                let _ = db.upsert_mention_job(&MentionJob {
                    id: None,
                    tweet_id: tweet_id.clone(),
                    user_handle: author_handle.clone(),
                    target_url: "N/A".to_string(),
                    editor_job_id: None,
                    status: "conversation".to_string(),
                    s3_video_url: None,
                    x_reply_id: None,
                    tweet_text: Some(text.clone()),
                    created_at: None,
                    updated_at: None,
                });
            }
        }
    }

    println!(
        "\n=== [INBOX COMPLETE] New Mentions: {}, Jobs Triggered: {} ===",
        new_count, jobs_count
    );
    Ok((new_count, jobs_count))
}
