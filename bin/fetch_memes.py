#!/usr/bin/env python3
"""
fetch_memes.py — Autonomous scraper to fetch real, trending tech/developer memes
from 4chan /g/ (Technology) and Reddit (r/ProgrammerHumor, r/techmemes).
Saves images locally in assets/memes/ and outputs JSON metadata for the dashboard and bot.
"""

import os
import sys
import json
import urllib.request
import re
import time

REPO_DIR = "/home/adnan/x_bot"
MEME_DIR = os.path.join(REPO_DIR, "assets", "memes")
CACHE_FILE = os.path.join(REPO_DIR, "data", "fetched_memes.json")
os.makedirs(MEME_DIR, exist_ok=True)
os.makedirs(os.path.dirname(CACHE_FILE), exist_ok=True)

USER_AGENT = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"

def fetch_reddit_memes(subreddits=["ProgrammerHumor", "techmemes"], limit=5):
    memes = []
    for sub in subreddits:
        url = f"https://meme-api.com/gimme/{sub}/{limit}"
        try:
            req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(req, timeout=8) as resp:
                data = json.loads(resp.read().decode())
                for m in data.get("memes", []):
                    img_url = m.get("url", "")
                    if not img_url or m.get("nsfw", False):
                        continue
                    if not (img_url.endswith(".png") or img_url.endswith(".jpg") or img_url.endswith(".jpeg") or img_url.endswith(".webp")):
                        continue

                    title = m.get("title", "")
                    # Generate natural builder bridge
                    caption = generate_meme_caption(title, "reddit")
                    memes.append({
                        "id": f"reddit_{m.get('postLink', '').split('/')[-1]}",
                        "title": title,
                        "source": f"r/{sub}",
                        "sourceUrl": m.get("postLink", ""),
                        "imageUrl": img_url,
                        "ups": m.get("ups", 0),
                        "caption": caption,
                        "format": "image_meme"
                    })
        except Exception as e:
            print(f"[Warn] Reddit fetch failed for {sub}: {e}")
    return memes

def fetch_4chan_memes(board="g", limit=6):
    memes = []
    catalog_url = f"https://a.4cdn.org/{board}/catalog.json"
    keywords = ["vibe", "ai", "coding", "screen", "demo", "loom", "code", "dev", "claude", "cursor", "browser", "model", "git", "linux", "saas", "tech", "lmg", "vcg"]
    
    try:
        req = urllib.request.Request(catalog_url, headers={
            "User-Agent": USER_AGENT,
            "Referer": "https://boards.4channel.org/"
        })
        with urllib.request.urlopen(req, timeout=8) as resp:
            catalog = json.loads(resp.read().decode())
            
            for page in catalog:
                for thread in page.get("threads", []):
                    # Skip sticky rules / global threads
                    if thread.get("sticky") or thread.get("closed"):
                        continue

                    tim = thread.get("tim")
                    ext = thread.get("ext", "")
                    if not (tim and ext in [".jpg", ".png", ".webp"]):
                        continue

                    sub = thread.get("sub", "")
                    com = thread.get("com", "")
                    clean_com = re.sub(r"<[^>]+>", " ", com).strip()
                    thread_text = f"{sub} {clean_com}".lower()

                    # Filter for real tech/coding threads
                    if any(k in thread_text for k in keywords) or (sub and len(sub) > 4):
                        img_url = f"https://i.4cdn.org/{board}/{tim}{ext}"
                        title = sub if sub else (clean_com[:60] if clean_com else f"4chan /{board}/ Tech Discussion")
                        
                        caption = generate_meme_caption(title, "4chan")
                        memes.append({
                            "id": f"4chan_{thread.get('no')}",
                            "title": title,
                            "source": f"4chan /{board}/",
                            "sourceUrl": f"https://boards.4chan.org/{board}/thread/{thread.get('no')}",
                            "imageUrl": img_url,
                            "ups": thread.get("replies", 0),
                            "caption": caption,
                            "format": "image_meme"
                        })
                        if len(memes) >= limit:
                            return memes
    except Exception as e:
        print(f"[Warn] 4chan fetch failed for /{board}/: {e}")
    return memes

def generate_meme_caption(title, source):
    t_lower = title.lower()
    
    # 1. Broken / janky / it works / duct tape memes
    if any(k in t_lower for k in ["works", "work", "slop", "broken", "fix", "tape", "jank", "hate software"]):
        options = [
            "my production backend held together by 3 edge functions, zero error handling, and sheer vibes. somehow hasn't crashed yet",
            "the screen recording setup you hack together with 4 virtual audio cables before giving up and using @trypitchdotco",
            "average tech stack in 2026: 15 ai wrappers, 1 supabase instance, and a founder on their 40th screen studio retake",
            "when your code is absolute spaghetti but the stripe webhooks are hitting and users are paying"
        ]
    # 2. GitHub / Git / Commits / PRs / Deploying
    elif any(k in t_lower for k in ["github", "git", "commit", "merge", "push", "branch", "repo"]):
        options = [
            "bro has 0 public commits, 14 unmerged local branches, and just deployed a $40k mrr app to production",
            "pushing directly to main at 4:59 pm on a friday and immediately closing the laptop",
            "building in public until you see your own messy codebase on the screen recording"
        ]
    # 3. AI / Claude / Cursor / Vibe Coding / LLMs
    elif any(k in t_lower for k in ["ai", "claude", "cursor", "vibe", "model", "anthropic", "openai", "agent"]):
        options = [
            "spent 10 minutes vibe-coding an entire full-stack saas with claude and then spent 4 business days trying to record a clean 60s demo video",
            "ai will replace software engineers by 2027 but founders will still be doing 50 screen studio takes because their dog barked at 0:58",
            "polymarket 95% probability devs spend more time arguing about AI models on twitter than writing code"
        ]
    # 4. Programming languages / Python / Rust / C++
    elif any(k in t_lower for k in ["python", "rust", "cpp", "c++", "javascript", "typescript", "linux"]):
        options = [
            "senior dev spending 6 hours debugging a memory leak only to realize the env variable was misspelled",
            "rewriting your entire backend in rust because you didn't want to record a 1-minute loom for your users",
            "why are programmers like this. 4 hours optimizing a sql query to save 2ms instead of shipping the demo"
        ]
    # 5. Friday / Deadlines / Stress / Late night
    elif any(k in t_lower for k in ["friday", "deadline", "four", "thirty", "night", "sleep", "stress"]):
        options = [
            "launching in 2 hours with 0 docs and a prayer. tag @trypitchdotco with your link and at least your demo video won't look like a 2012 screencast",
            "it is 3:30 am, the app is broken, and you are still editing zooms in premiere pro. just let @trypitchdotco render the video in 60s",
            "the Friday deployment energy. what could possibly go wrong"
        ]
    # 6. Tech support / Family / Hardware / General Tech Twitter Pain
    elif any(k in t_lower for k in ["computer", "phone", "hardware", "printer", "nokia", "apple"]):
        options = [
            "paying $1,600 for a phone just to check stripe dashboard and argue with strangers on tech twitter",
            "can you fix my wifi? no auntie i build AI agents that turn product URLs into 60s narrated video demos, your printer is beyond help",
            "the reality of working in tech vs what your family thinks you do"
        ]
    # 7. Sarcastic Builder Defaults (Relatable rage-bait / founder banter)
    else:
        options = [
            "nothing tests founder sanity like doing take #34 of a product video and getting a slack ping mid-zoom. @trypitchdotco fixes this in 1 prompt",
            "building the product: 2 days. recording a clean 60s demo without stuttering: 2 weeks. stop suffering and generate it on @trypitchdotco",
            "polymarket 90% odds that founders spend more time re-recording 2-minute product demos than writing code",
            "the 4 stages of recording a demo: 1. this takes 5 mins 2. mic was muted 3. notification popup 4. it is 4 am and you're learning after effects"
        ]

    import random
    return random.choice(options)

def download_meme_images(memes):
    downloaded = []
    for m in memes:
        try:
            ext = m["imageUrl"].split(".")[-1].split("?")[0]
            local_filename = f"{m['id']}.{ext}"
            local_path = os.path.join(MEME_DIR, local_filename)

            if not os.path.exists(local_path):
                headers = {"User-Agent": USER_AGENT}
                if "4cdn.org" in m["imageUrl"]:
                    headers["Referer"] = "https://boards.4channel.org/"
                req = urllib.request.Request(m["imageUrl"], headers=headers)
                with urllib.request.urlopen(req, timeout=10) as resp:
                    with open(local_path, "wb") as f:
                        f.write(resp.read())

            m["localPath"] = local_path
            downloaded.append(m)
        except Exception as e:
            print(f"[Warn] Download failed for {m['id']}: {e}")
            downloaded.append(m)
    return downloaded

def main():
    print("=== [FETCHING FRESH TECH & SAAS MEMES] ===")
    reddit_memes = fetch_reddit_memes(limit=4)
    chan_memes = fetch_4chan_memes(board="g", limit=4)

    all_memes = reddit_memes + chan_memes
    processed = download_meme_images(all_memes)

    with open(CACHE_FILE, "w") as f:
        json.dump(processed, f, indent=2)

    print(f"[OK] Fetched & cached {len(processed)} real tech memes from Reddit & 4chan!")
    for m in processed:
        print(f"  • [{m['source']}] {m['title'][:45]} -> {m['imageUrl']}")

if __name__ == "__main__":
    main()
