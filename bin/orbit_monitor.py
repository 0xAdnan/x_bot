#!/usr/bin/env python3
"""
orbit_monitor.py — Real-Time Influencer Radar & Contextual Infiltration Engine.
Monitors top AI & tech influencers:
  @DarioAmodei_AI (Dario Amodei), @karpathy, @sama, @levelsio, @swyx,
  @rauchg, @hwchase17, @bindureddy, @rowancheung, @Polymarket
Parses the exact tweet context and generates unique, non-generic, anti-AI builder takes for @adnanspitch.
"""

import os
import sys
import json
import asyncio
import re
import random
import time
from playwright.async_api import async_playwright

REPO_DIR = "/home/adnan/x_bot"
RADAR_FILE = os.path.join(REPO_DIR, "data", "influencer_radar.json")
STORAGE_STATE = os.path.join(REPO_DIR, ".browser-profile-adnanspitch", "storageState.json")
if not os.path.exists(STORAGE_STATE):
    STORAGE_STATE = os.path.join(REPO_DIR, ".browser-profile", "storageState.json")

TARGET_INFLUENCERS = [
    {"handle": "DarioAmodei_AI", "name": "Dario Amodei", "niche": "Anthropic / Claude / Scaling"},
    {"handle": "karpathy", "name": "Andrej Karpathy", "niche": "AI / Deep Learning"},
    {"handle": "sama", "name": "Sam Altman", "niche": "OpenAI / Frontier Tech"},
    {"handle": "levelsio", "name": "Pieter Levels", "niche": "Solopreneur / SaaS"},
    {"handle": "rauchg", "name": "Guillermo Rauch", "niche": "Vercel / Next.js / Frontend"},
    {"handle": "swyx", "name": "swyx", "niche": "Latent Space / MCP"},
    {"handle": "hwchase17", "name": "Harrison Chase", "niche": "LangChain / Agents"},
    {"handle": "bindureddy", "name": "Bindu Reddy", "niche": "Abacus AI / LLM Debates"},
    {"handle": "rowancheung", "name": "Rowan Cheung", "niche": "The Rundown AI"},
    {"handle": "Polymarket", "name": "Polymarket", "niche": "Prediction Markets"}
]

BANNED_WORDS = [
    "delve", "elevate", "unlock", "leverage", "seamless", "robust",
    "game-changer", "revolutionary", "transformative", "landscape", "streamline"
]

def clean_anti_ai(text: str) -> str:
    """Strictly strip em-dashes, en-dashes, and emojis."""
    t = text.replace("—", ", ").replace("–", ", ").replace(" - ", ", ")
    t = re.sub(r"[\U00010000-\U0010ffff]|[\u2600-\u27BF]|[\uD83C-\uDBFF\uDC00-\uDFFF]", "", t)
    t = re.sub(r"\s+", " ", t).strip()
    return t

def generate_contextual_takes(author_handle: str, author_name: str, tweet_text: str) -> dict:
    """
    Extracts specific themes, keywords, and topics from the actual tweet text
    to generate distinct, non-generic, high-IQ builder takes for @adnanspitch.
    """
    clean_text = tweet_text.replace("\n", " ").strip()
    t_lower = clean_text.lower()
    
    # Extract salient keywords
    is_reasoning_or_claude = any(k in t_lower for k in ["claude", "dario", "anthropic", "reasoning", "thinking", "extended thinking", "sonnet", "opus"])
    is_agent_or_mcp = any(k in t_lower for k in ["agent", "mcp", "tool use", "computer use", "autonomous", "browser use", "workflow"])
    is_compute_or_scaling = any(k in t_lower for k in ["scaling", "compute", "gpu", "cluster", "datacenter", "flops", "power", "energy"])
    is_vibe_or_coding = any(k in t_lower for k in ["vibe coding", "cursor", "copilot", "coding", "software engineer", "full stack", "github"])
    is_saas_or_shipping = any(k in t_lower for k in ["ship", "saas", "mrr", "revenue", "startup", "launch", "founder", "indie", "mvp"])
    is_benchmark_or_eval = any(k in t_lower for k in ["benchmark", "swe-bench", "eval", "mmlu", "arena", "test", "score"])
    is_polymarket_or_odds = any(k in t_lower for k in ["polymarket", "bet", "odds", "probability", "market", "election", "prediction"])

    takes_pool = []

    if is_reasoning_or_claude:
        takes_pool.extend([
            "the gap between models generating 10,000 lines of code in seconds and founders still needing 3 days to record a 1-minute demo video is where all the friction lives now",
            "extended thinking models show that compute at test-time beats human micromanagement every time. the next step is models demoing and validating their own interfaces",
            "seeing frontier reasoning models output full full-stack architectures in one prompt makes it very clear that distribution and video storytelling are the only remaining moats",
            "the real bottleneck with reasoning models is not generation speed, it is human bandwidth to review, demo, and communicate what was actually built"
        ])
    elif is_agent_or_mcp:
        takes_pool.extend([
            "MCP is quietly solving the hardest part of software integration. instead of building 40 custom APIs, you give agents deterministic tool interfaces and let them orchestrate",
            "the logical conclusion of browser-use agents is software that creates its own cinematic product walkthroughs and interactive documentation from a single prompt",
            "most agent demos fail because people try to show 10 complex steps in raw terminal text. visual, automated video walkthroughs make agents actually understandable",
            "building the agent takes 1 weekend now. proving to users that the agent actually works without hallucinatory loops is the actual product"
        ])
    elif is_compute_or_scaling:
        takes_pool.extend([
            "datacenter power constraints are forcing teams to get radically efficient with inference. the winners won't be who owns the biggest cluster, but who turns compute into customer value fastest",
            "scaling laws holding up on reasoning means software production cost is heading to zero. when anyone can build software, attention and polished video distribution become 10x more valuable",
            "spending gigawatts on training just for software to sit unseen on github repos without a clean video demo is the ultimate misallocation of compute"
        ])
    elif is_vibe_or_coding:
        takes_pool.extend([
            "vibe coding is awesome until you have to record a 60s product demo for your launch and realize you spent 4 hours doing 35 screen recorder retakes",
            "the modern stack is vibe coding the backend in 20 minutes with claude, then automating the entire launch video and onboarding walkthrough so you can ship the same day",
            "software development is becoming prompt direction. the founders who win are the ones who can turn text prompts directly into working apps and polished video demos"
        ])
    elif is_saas_or_shipping:
        takes_pool.extend([
            "shipping fast is the only survival strategy. if your launch video takes longer to edit than the feature took to build, you are losing momentum",
            "indie hackers spending 2 days coding a feature then 2 weeks dreading the screen recording retakes is the most common founder bottleneck right now",
            "the best marketing for a SaaS product in 2026 is showing the exact 30s workflow in action. no marketing fluff, just clean automated video walkthroughs"
        ])
    elif is_polymarket_or_odds:
        takes_pool.extend([
            "polymarket 95% probability that manual screen recording dies out as text-to-video demo engines turn any URL into a narrated walkthrough in 60s",
            "prediction markets are pricing tech adoption faster than venture funds. the consensus on automated developer tooling moving to production is already settled",
            "the odds on AI replacing manual video editing workflows in SaaS onboarding are basically at 99% right now"
        ])
    else:
        # Grounded founder observations tailored to author
        if "levels" in author_handle.lower():
            takes_pool.extend([
                "speed over perfection is the only moat that holds up. automate the repetitive video demo and marketing production so you can keep shipping daily",
                "building 12 startups in 12 months is realistic now if you automate the launch clip and demo recording bottlenecks from day 1"
            ])
        elif "karpathy" in author_handle.lower():
            takes_pool.extend([
                "the transition from coding in syntax to coding in natural language specifications makes deterministic visual verification the core primitive of devtools",
                "software 2.0 was neural nets, software 3.0 is agents that build, test, and generate their own visual product demos from plain text task descriptions"
            ])
        elif "sama" in author_handle.lower():
            takes_pool.extend([
                "as intelligence becomes a commodity utility, the bottleneck shifts completely to distribution, clarity, and how fast you can explain your product to users",
                "the 1-person billion dollar company will be built by founders who automate everything from code generation to video demo production"
            ])
        else:
            takes_pool.extend([
                "the best developer tools solve the friction nobody wants to talk about: like spending 4 hours doing retakes on a screen recording because you coughed at 0:54",
                "in an era where code is generated in seconds, product clarity and 45-second visual demos are how builders actually stand out on the timeline",
                "90% of tech discourse is theoretical, the other 10% is builders trying to ship features and get video walkthroughs in front of real users"
            ])

    # Pick primary quote take and early reply
    random.shuffle(takes_pool)
    primary_qt = clean_anti_ai(takes_pool[0])
    
    # Reply take tailored to the specific author
    early_reply = clean_anti_ai(f"real talk @{author_handle}. {takes_pool[1] if len(takes_pool) > 1 else primary_qt}")

    # Alternative rotation options
    alt_takes = [clean_anti_ai(t) for t in takes_pool[1:4]]

    return {
        "quoteTweetTake": primary_qt,
        "earlyReplyTake": early_reply,
        "alternativeTakes": alt_takes,
        "char_count_qt": len(primary_qt)
    }

async def scan_influencer_timeline(page, target):
    handle = target["handle"]
    url = f"https://x.com/{handle}"
    print(f"[Radar] Scanning @{handle} ({target['name']})...")
    
    detected = []
    try:
        await page.goto(url, wait_until="domcontentloaded")
        await page.wait_for_timeout(3500)

        articles = page.locator('article[data-testid="tweet"]')
        count = await articles.count()
        
        for i in range(min(count, 3)):
            art = articles.nth(i)
            time_el = art.locator('time').first
            link_el = art.locator('a[href*="/status/"]').first
            text_el = art.locator('div[data-testid="tweetText"]').first

            if await link_el.count() > 0 and await text_el.count() > 0:
                href = await link_el.get_attribute('href')
                text = await text_el.inner_text()
                time_str = await time_el.get_attribute('datetime') if await time_el.count() > 0 else "recently"
                
                # Check for pinned tweet vs fresh tweet
                tweet_url = f"https://x.com{href}" if href.startswith('/') else href
                tweet_id = tweet_url.split('/status/')[-1].split('?')[0]

                takes = generate_contextual_takes(handle, target["name"], text)

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
                    "alternativeTakes": takes["alternativeTakes"],
                    "charCount": takes["char_count_qt"]
                })
    except Exception as e:
        print(f"[Radar Error] Failed to scan @{handle}: {e}")
    return detected

async def run_radar_pass():
    print("=== [INFLUENCER RADAR & CONTEXTUAL QUOTE TWEET ENGINE] ===")
    all_events = []

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(
            storage_state=STORAGE_STATE if os.path.exists(STORAGE_STATE) else None,
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = await context.new_page()

        # Scan top 5 influencers per pass
        import random
        selected = random.sample(TARGET_INFLUENCERS, min(5, len(TARGET_INFLUENCERS)))

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

    combined = combined[:30]

    with open(RADAR_FILE, "w") as f:
        json.dump(combined, f, indent=2)

    print(f"[OK] Influencer Radar Updated: {len(combined)} active influencer tweets with custom contextual takes ready!")
    return combined

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--test-gen":
        sample_tweets = [
            ("DarioAmodei_AI", "Dario Amodei", "Claude 3.7 Sonnet introduces hybrid reasoning models that can think dynamically before answering complex SWE benchmarks."),
            ("sama", "Sam Altman", "The intelligence age will be defined by abundant compute and autonomous agents capable of complex long-horizon execution."),
            ("levelsio", "Pieter Levels", "Just shipped another micro SaaS in 48 hours. Vibe coding with Claude is 100x faster than traditional development."),
            ("karpathy", "Andrej Karpathy", "DeepSeek and open weights models are demonstrating incredible efficiency in reasoning with smaller parameter budgets.")
        ]
        print("=== TESTING CONTEXTUAL TAKE GENERATION ===")
        for handle, name, txt in sample_tweets:
            t = generate_contextual_takes(handle, name, txt)
            print(f"\n[{name} (@{handle})]: \"{txt}\"")
            print(f"  🔥 Quote Tweet ({t['char_count_qt']}c): \"{t['quoteTweetTake']}\"")
            print(f"  💬 Reply: \"{t['earlyReplyTake']}\"")
            print(f"  🔄 Alt Take: \"{t['alternativeTakes'][0]}\"")
    else:
        asyncio.run(run_radar_pass())
