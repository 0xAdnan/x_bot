use crate::{
    db::{Database, Prospect},
    x_api::XApiClient,
};
use regex::Regex;

const DEFAULT_QUERIES: &[&str] = &[
    "(@ProductHunt OR \"Product Hunt\") (launching OR \"launching soon\" OR \"live today\" OR \"product of the day\")",
    "(\"launching on product hunt\" OR \"launching next week\" OR \"launch day\") (SaaS OR AI OR devtool)",
    "tella.tv",
    "screen.studio",
    "loom.com alternative",
    "need a product demo video",
    "how to make a product demo video",
    "(\"launching today\" OR \"just launched\") (SaaS OR AI OR devtool)",
    "(\"YC S26\" OR \"YC W26\" OR \"YC S25\")",
    "(\"building in public\" OR \"indie hacker\") (demo OR launch OR video)",
];

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

fn calculate_lead_score(text: &str, _handle: &str, url: &str) -> i32 {
    let mut score = 5;
    if url != "N/A" {
        score += 2;
    }
    let lower = text.to_lowercase();
    if ["tella", "screen studio", "loom", "guidde", "supademo", "tango"]
        .iter()
        .any(|k| lower.contains(k))
    {
        score += 2;
    }
    if ["product hunt", "producthunt", "ph launch", "launching soon", "launching today"]
        .iter()
        .any(|k| lower.contains(k))
    {
        score += 3;
    }
    if ["need", "looking for", "alternative", "how to", "launched", "building"]
        .iter()
        .any(|k| lower.contains(k))
    {
        score += 1;
    }
    score.min(10)
}

fn generate_pitch_hook(handle: &str, url: &str, text_context: &str) -> String {
    let clean_handle = handle.replace('@', "").to_lowercase();
    let lower_context = text_context.to_lowercase();
    let clean_url = url.replace("https://", "").replace("http://", "").replace("www.", "");
    let domain_clean = clean_url.split('/').next().unwrap_or("your app");

    if lower_context.contains("product hunt") || lower_context.contains("producthunt") {
        format!(
            "congrats on the product hunt launch @{}. if you need a quick 45s launch video walkthrough for {}, tag @trypitchdotco with your link and we'll render one for you",
            clean_handle, domain_clean
        )
    } else if lower_context.contains("screen studio") || lower_context.contains("quiro") {
        format!(
            "yo @{}, saw your post on screen recording. threw together a quick 60s walkthrough on @trypitchdotco to test the pacing, thought it looked super clean",
            clean_handle
        )
    } else if lower_context.contains("loom") {
        format!(
            "saw u were looking for a loom alt with better aesthetics. made a quick 60s video of {} on @trypitchdotco to show how the auto-zooms look, check it out if u want",
            domain_clean
        )
    } else if lower_context.contains("launch") || lower_context.contains("yc") {
        format!(
            "congrats on shipping @{}. cooked up a 45s launch video walkthrough of {} using @trypitchdotco, thought you might like it for your docs/feed",
            clean_handle, domain_clean
        )
    } else if url != "N/A" && !url.is_empty() {
        format!(
            "hey @{}, checked out {} on my feed. genuinely clean product. made a quick 60s narrated demo on @trypitchdotco to see how it looks in action",
            clean_handle, domain_clean
        )
    } else {
        format!(
            "hey @{}, saw your thread about shipping. we automated the whole 60s narrated demo video headache on @trypitchdotco, let me know if u want a free walkthrough for your app",
            clean_handle
        )
    }
}

pub async fn discover_prospects(
    max_per_query: usize,
    dry_run: bool,
) -> Result<usize, String> {
    println!("=== [DISCOVERING ICP PROSPECTS VIA X API SEARCH] ===");

    let db = Database::open().map_err(|e| format!("DB Error: {}", e))?;
    let mut x_client = XApiClient::new();

    let mut discovered_count = 0;

    for &query in DEFAULT_QUERIES {
        println!("\n[Search Query]: \"{}\"...", query);
        match x_client.search_recent(query, max_per_query).await {
            Ok(results) => {
                for item in results {
                    let handle = item["author"].as_str().unwrap_or_default().to_string();
                    if handle.is_empty()
                        || handle.eq_ignore_ascii_case("@trypitchdotco")
                        || handle.eq_ignore_ascii_case("@adnanspitch")
                    {
                        continue;
                    }

                    if let Ok(Some(_)) = db.get_prospect_by_handle(&handle) {
                        continue;
                    }

                    let text = item["text"].as_str().unwrap_or_default();
                    let name = item["name"].as_str().unwrap_or(&handle);
                    let target_url = extract_url(text);
                    let score = calculate_lead_score(text, &handle, &target_url);
                    let hook = generate_pitch_hook(&handle, &target_url, text);

                    println!(
                        "  -> Discovered Lead: {} | Target URL: {} | Score: {}/10",
                        handle, target_url, score
                    );

                    if dry_run {
                        println!("     [DRY RUN] Hook: {}", hook);
                        continue;
                    }

                    let p = Prospect {
                        id: None,
                        handle: handle.clone(),
                        name: Some(name.to_string()),
                        url: None,
                        segment: Some("founder".to_string()),
                        score: Some(score),
                        stage: Some("new".to_string()),
                        last_touch: None,
                        next_action_date: None,
                        touches: Some(0),
                        product_url: if target_url != "N/A" {
                            Some(target_url)
                        } else {
                            None
                        },
                        last_variant: None,
                        outcome: None,
                        notes: Some(format!("Discovered via search '{}'. Hook: {}", query, hook)),
                        why: Some(format!(
                            "Matched query '{}' with text: {}",
                            query,
                            text.chars().take(80).collect::<String>()
                        )),
                        updated_at: None,
                    };

                    let _ = db.upsert_prospect(&p);
                    discovered_count += 1;
                }
            }
            Err(e) => println!("[Search Warning] Query '{}' failed: {}", query, e),
        }
    }

    println!(
        "\n=== [DISCOVERY COMPLETE] Total New Prospects Saved: {} ===",
        discovered_count
    );
    Ok(discovered_count)
}
