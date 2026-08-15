#!/usr/bin/env python3
"""
orbit_monitor.py — Real-Time Influencer Radar & Early Infiltration Daemon.
Monitors top AI & tech influencers (@levelsio, @karpathy, @sama, @rowancheung, @Polymarket, @swyx, @bindureddy).
Detects fresh tweets within minutes and generates witty Quote-Tweet and Early Reply takes.
"""

import os
import sys
import json
import asyncio
import re
import time
from playwright.async_api import async_playwright

REPO_DIR = "/home/adnan/x_bot"
RADAR_FILE = os.path.join(REPO_DIR, "data", "influencer_radar.json")
STORAGE_STATE = os.path.join(REPO_DIR, ".browser-profile-adnanspitch", "storageState.json")
if not os.path.exists(STORAGE_STATE):
    STORAGE_STATE = os.path.join(REPO_DIR, ".browser-profile", "storageState.json")

TARGET_INFLUENCERS = [
    {"handle": "levelsio", "name": "Pieter Levels", "niche": "Solopreneur / SaaS"},
    {"handle": "karpathy", "name": "Andrej Karpathy", "niche": "AI / Deep Learning"},
    {"handle": "sama", "name": "Sam Altman", "niche": "OpenAI / Frontier Tech"},
    {"handle": "rowancheung", "name": "Rowan Cheung", "niche": "The Rundown AI"},
    {"handle": "Polymarket", "name": "Polymarket", "niche": "Prediction Markets"},
    {"handle": "swyx", "name": "swyx", "niche": "Latent Space / MCP"},
    {"handle": "bindureddy", "name": "Bindu Reddy", "niche": "Abacus AI / LLM Debates"}
]

def generate_witty_takes(author_handle, tweet_text):
    clean_text = tweet_text.replace("\n", " ").strip()
    t_lower = clean_text.lower()
    
    # 1. Quote Tweet Take for @adnanspitch (Witty, sharp observation, 0 promo)
    if "agent" in t_lower or "model" in t_lower or "claude" in t_lower or "reasoning" in t_lower:
        qt_take = "the pace of agent capability is compounding faster than people realize. the bottleneck isn't intelligence anymore, it's UX and workflow integrations."
        reply_take = "accurate. the gap between frontier model capabilities and how everyday builders actually interface with them is where the real value is right now."
    elif "ship" in t_lower or "saas" in t_lower or "build" in t_lower or "code" in t_lower or "vibe" in t_lower:
        qt_take = "shipping speed is the only moat that actually holds up in 2026. if you take 3 weeks to launch a feature you've already lost."
        reply_take = "100%. the builders winning right now are the ones who turned their feedback-to-deploy loop into a 1-day turnaround."
    elif "polymarket" in t_lower or "bet" in t_lower or "odds" in t_lower:
        qt_take = "prediction markets are lowkey the most efficient truth engine on the internet right now. crowd pricing beats pundit takes every single time."
        reply_take = "the odds spread on tech adoption is always where the signal is. wild to see how fast consensus shifts."
    else:
        qt_take = "fascinating angle on this. seeing this exact pattern play out across devtools and indie founders shipping this quarter."
        reply_take = "great observation. the shift over the last 6 months has been completely non-linear."

    # 2. Contextual Brand Bridge for @trypitchdotco
    brand_bridge = "clean perspective. we've seen this exact pain point firsthand: builders spending days struggling with video walkthroughs instead of shipping features."

    return {
        "quoteTweetTake": qt_take,
        "earlyReplyTake": reply_take,
        "brandBridge": brand_bridge
    }

async def scan_influencer_timeline(page, target):
    handle = target["handle"]
    url = f"https://x.com/{handle}"
    print(f"[Radar] Scanning @{handle}...")
    
    detected = []
    try:
        await page.goto(url, wait_until="domcontentloaded")
        await page.wait_for_timeout(3500)

        # Get first 3 tweets
        articles = page.locator('article[data-testid="tweet"]')
        count = await articles.count()
        
        for i in range(min(count, 3)):
            art = articles.nth(i)
            # Extract tweet link
            time_el = art.locator('time').first
            link_el = art.locator('a[href*="/status/"]').first
            text_el = art.locator('div[data-testid="tweetText"]').first

            if await link_el.count() > 0 and await text_el.count() > 0:
                href = await link_el.get_attribute('href')
                text = await text_el.inner_text()
                time_str = await time_el.get_attribute('datetime') if await time_el.count() > 0 else "recently"
                
                tweet_url = f"https://x.com{href}" if href.startswith('/') else href
                tweet_id = tweet_url.split('/status/')[-1].split('?')[0]

                takes = generate_witty_takes(handle, text)

                detected.append({
                    "id": f"radar_{tweet_id}",
                    "author": f"@{handle}",
                    "authorName": target["name"],
                    "niche": target["niche"],
                    "tweetUrl": tweet_url,
                    "text": text,
                    "time": time_str,
                    "quoteTweetTake": takes["quoteTweetTake"],
                    "earlyReplyTake": takes["earlyReplyTake"],
                    "brandBridge": takes["brandBridge"]
                })
    except Exception as e:
        print(f"[Radar Error] Failed to scan @{handle}: {e}")
    return detected

async def run_radar_pass():
    print("=== [INFLUENCER RADAR & EARLY INFILTRATION SCAN] ===")
    all_events = []

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(
            storage_state=STORAGE_STATE if os.path.exists(STORAGE_STATE) else None,
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = await context.new_page()

        # Scan top 4 influencers per pass to keep passes fast (under 30s)
        import random
        selected = random.sample(TARGET_INFLUENCERS, min(4, len(TARGET_INFLUENCERS)))

        for inf in selected:
            evs = await scan_influencer_timeline(page, inf)
            all_events.extend(evs)
            await page.wait_for_timeout(1500)

        await browser.close()

    # Merge with existing radar file if present
    existing = []
    if os.path.exists(RADAR_FILE):
        try:
            with open(RADAR_FILE, "r") as f:
                existing = json.load(f)
        except Exception:
            existing = []

    # Dedup by tweetUrl
    seen_urls = set()
    combined = []
    for item in all_events + existing:
        if item["tweetUrl"] not in seen_urls:
            seen_urls.add(item["tweetUrl"])
            combined.append(item)

    # Keep latest 25 radar items
    combined = combined[:25]

    with open(RADAR_FILE, "w") as f:
        json.dump(combined, f, indent=2)

    print(f"[OK] Influencer Radar Updated: {len(combined)} active high-heat influencer tweets ready for infiltration!")
    return combined

if __name__ == "__main__":
    asyncio.run(run_radar_pass())
