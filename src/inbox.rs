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
    MissingUrlRequest,           // User asked for a demo/video or how to trigger one, but NO URL was provided
    TechQuestion,                // User is asking technical questions about architecture/stack
    PraiseOrGreeting,            // User is saying thanks, good job, congrats, or hi
    Conversation,                // General chat
}

pub async fn resolve_url(text: &str) -> String {
    let raw_url = extract_url(text);
    if raw_url == "N/A" {
        return raw_url;
    }
    if raw_url.contains("t.co/") {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .redirect(reqwest::redirect::Policy::limited(5))
            .timeout(std::time::Duration::from_secs(5))
            .build();
        if let Ok(c) = client {
            if let Ok(resp) = c.get(&raw_url).send().await {
                let final_u = resp.url().as_str().to_string();
                if !final_u.is_empty() && !final_u.contains("t.co/") {
                    return final_u;
                }
                if let Ok(body) = resp.text().await {
                    let re_meta = Regex::new(r#"(?i)URL=(https?://[^\s"'>]+)"#).unwrap();
                    if let Some(caps) = re_meta.captures(&body) {
                        if let Some(m) = caps.get(1) {
                            return m.as_str().to_string();
                        }
                    }
                    let re_title = Regex::new(r#"(?i)<title>(https?://[^\s"'>]+)</title>"#).unwrap();
                    if let Some(caps) = re_title.captures(&body) {
                        if let Some(m) = caps.get(1) {
                            return m.as_str().to_string();
                        }
                    }
                }
            }
        }
    }
    raw_url
}

pub async fn classify_mention_intent(text: &str) -> MentionIntent {
    let lower = text.to_lowercase();
    let url = resolve_url(text).await;

    let is_valid_product_url = url != "N/A"
        && !url.contains("trypitch.co")
        && !url.contains("twitter.com")
        && !url.contains("x.com")
        && !url.contains("t.co/")
        && !url.contains("localhost")
        && !url.contains("loca.lt")
        && !url.contains("vercel.app");

    let launch_triggers = [
        "launch video", "launch demo", "product hunt launch", "for launch",
        "make a launch", "create a launch", "generate a launch",
        "build a launch", "launch walkthrough", "launch clip", "launching"
    ];

    let demo_triggers = [
        "make a demo", "create a demo", "generate a demo", "record a demo",
        "make a video", "create a video", "generate a video", "record a video",
        "show me a demo", "build a demo", "demo for", "demo of",
        "walkthrough for", "walkthrough of", "can you make", "can you create",
        "can you demo", "can you record", "make me a", "generate me a",
        "cook a demo", "cook a video", "video for", "demo this",
        "video demo", "make demo", "generate demo", "give me a demo", "give me a video",
        "demo video", "walkthrough", "product demo"
    ];

    if is_valid_product_url {
        if launch_triggers.iter().any(|t| lower.contains(t)) {
            return MentionIntent::LaunchVideo(url);
        }
        if demo_triggers.iter().any(|t| lower.contains(t)) {
            return MentionIntent::DemoVideo(url);
        }
        return MentionIntent::ConversationWithUrl(url);
    }

    // NO URL PROVIDED:
    // 1. User asked for a video / demo or how to use the bot without providing a URL
    if demo_triggers.iter().any(|t| lower.contains(t))
        || launch_triggers.iter().any(|t| lower.contains(t))
        || lower.contains("how to use")
        || lower.contains("how do i use")
        || lower.contains("how to get a video")
        || (lower.contains("how does it work") && (lower.contains("try") || lower.contains("demo")))
    {
        return MentionIntent::MissingUrlRequest;
    }

    // 2. Technical question
    if lower.contains("stack")
        || lower.contains("tech")
        || lower.contains("how does this work")
        || lower.contains("how it works")
        || lower.contains("under the hood")
        || lower.contains("architecture")
        || lower.contains("playwright")
        || lower.contains("model")
        || lower.contains("agent")
    {
        return MentionIntent::TechQuestion;
    }

    // 3. Praise or Greeting
    if lower.contains("thank")
        || lower.contains("nice")
        || lower.contains("cool")
        || lower.contains("awesome")
        || lower.contains("fire")
        || lower.contains("love")
        || lower.contains("congrat")
        || lower.contains("congrats")
        || lower.contains("great job")
        || lower.contains("insane")
        || lower.contains("sick")
        || lower.contains("gm")
        || lower.contains("hello")
        || (lower.contains("yo") && lower.len() < 20)
    {
        return MentionIntent::PraiseOrGreeting;
    }

    // 4. General conversation
    MentionIntent::Conversation
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
        let intent = classify_mention_intent(&text).await;

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
                        "on it @{}. cooking up a launch video for {} right now, will drop the link right here when it's done",
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
                    "checked out {} @{}, looks super clean. let us know if you ever want a 60s narrated video walkthrough or launch demo for it",
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

            MentionIntent::MissingUrlRequest => {
                println!("[INTENT: MISSING URL REQUEST] User: {} asked for video/instructions without URL", author_handle);
                let clean_user = author_handle.replace('@', "").to_lowercase();
                let reply_msg = format!(
                    "hey @{}, drop your product link in the mention (like @trypitchdotco make a demo for example.com) and we'll render a 60s narrated walkthrough for you",
                    clean_user
                );
                if !dry_run {
                    match x_client.post_tweet(&reply_msg, Some(&tweet_id)).await {
                        Ok(rid) => println!("[Missing URL Reply Sent] Reply Tweet ID: {}", rid),
                        Err(e) => println!("[Missing URL Reply Warning]: {}", e),
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

            MentionIntent::TechQuestion => {
                println!("[INTENT: TECH QUESTION] User: {} asking about architecture/stack", author_handle);
                let clean_user = author_handle.replace('@', "").to_lowercase();
                let reply_msg = format!(
                    "hey @{}, we use automated browser recording + vision models to script the flow, synthetic narration for voiceover, and a motion engine to render the 1080p mp4 in ~60s",
                    clean_user
                );
                if !dry_run {
                    match x_client.post_tweet(&reply_msg, Some(&tweet_id)).await {
                        Ok(rid) => println!("[Tech Question Reply Sent] Reply Tweet ID: {}", rid),
                        Err(e) => println!("[Tech Question Reply Warning]: {}", e),
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

            MentionIntent::PraiseOrGreeting => {
                println!("[INTENT: PRAISE OR GREETING] User: {}", author_handle);
                let lower_text = text.to_lowercase();
                let clean_user = author_handle.replace('@', "").to_lowercase();
                let reply_msg = if lower_text.contains("thank") || lower_text.contains("nice") || lower_text.contains("cool") || lower_text.contains("awesome") || lower_text.contains("fire") || lower_text.contains("love") || lower_text.contains("congrat") || lower_text.contains("congrats") || lower_text.contains("great") || lower_text.contains("sick") || lower_text.contains("insane") {
                    format!("appreciate the love @{}! let us know anytime if you want a walkthrough for anything you're shipping", clean_user)
                } else {
                    format!("hey @{}, good to see you on the timeline! hope building is going well", clean_user)
                };
                if !dry_run {
                    match x_client.post_tweet(&reply_msg, Some(&tweet_id)).await {
                        Ok(rid) => println!("[Praise/Greeting Reply Sent] Reply Tweet ID: {}", rid),
                        Err(e) => println!("[Praise/Greeting Reply Warning]: {}", e),
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

            MentionIntent::Conversation => {
                println!("[INTENT: CONVERSATIONAL CHAT] User: {} (No video requested - DO NOT call Pitch MCP)", author_handle);
                let lower_text = text.to_lowercase();
                let clean_user = author_handle.replace('@', "").to_lowercase();

                let reply_msg = if lower_text.contains("connect") || lower_text.contains("dm") || lower_text.contains("chat") {
                    format!("sounds good @{}, dms are open anytime", clean_user)
                } else {
                    format!("hey @{}, good to see you on the timeline. let us know anytime if you want a quick product walkthrough", clean_user)
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
