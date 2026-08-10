use crate::{
    db::{Database, Prospect},
    x_api::XApiClient,
};
use regex::Regex;

const DEFAULT_QUERIES: &[&str] = &[
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
    if ["need", "looking for", "alternative", "how to", "launched", "building"]
        .iter()
        .any(|k| lower.contains(k))
    {
        score += 1;
    }
    score.min(10)
}

fn generate_pitch_hook(handle: &str, url: &str) -> String {
    if url != "N/A" {
        format!(
            "Hey {}, saw your post regarding {}! Created a 60s AI video demo walkthrough using PITCH — check it out!",
            handle, url
        )
    } else {
        format!(
            "Hey {}, saw you discussing SaaS video demos! PITCH automatically generates 1080p narrated product walkthroughs from any URL in 60s.",
            handle
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
                    let hook = generate_pitch_hook(&handle, &target_url);

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
