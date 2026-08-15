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
    keywords = ["vibe", "ai", "coding", "screen", "demo", "loom", "code", "dev", "claude", "cursor", "browser", "model", "git", "linux", "saas", "tech"]
    
    try:
        req = urllib.request.Request(catalog_url, headers={"User-Agent": USER_AGENT})
        with urllib.request.urlopen(req, timeout=8) as resp:
            catalog = json.loads(resp.read().decode())
            
            for page in catalog:
                for thread in page.get("threads", []):
                    tim = thread.get("tim")
                    ext = thread.get("ext", "")
                    if not (tim and ext in [".jpg", ".png", ".webp"]):
                        continue

                    sub = thread.get("sub", "")
                    com = thread.get("com", "")
                    clean_com = re.sub(r"<[^>]+>", " ", com).strip()
                    thread_text = f"{sub} {clean_com}".lower()

                    # Filter for tech/coding relevance
                    if any(k in thread_text for k in keywords) or not sub:
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
    title_clean = title.replace("\n", " ").strip()
    if len(title_clean) > 80:
        title_clean = title_clean[:77] + "..."

    templates = [
        f"\"{title_clean}\" — why are we still suffering through manual screen demo recording in 2026. tag @trypitchdotco with your link and we'll render a 60s narrated walkthrough for you",
        f"the exact founder mood when you do 40 takes on screen studio because your notification popped up at 0:58. @trypitchdotco fixes this in 1 prompt",
        f"\"{title_clean}\" — 90% of SaaS launch friction is making the product demo video. stop editing keyframes and generate it with @trypitchdotco"
    ]
    import random
    return random.choice(templates)

def download_meme_images(memes):
    downloaded = []
    for m in memes:
        try:
            ext = m["imageUrl"].split(".")[-1].split("?")[0]
            local_filename = f"{m['id']}.{ext}"
            local_path = os.path.join(MEME_DIR, local_filename)

            if not os.path.exists(local_path):
                req = urllib.request.Request(m["imageUrl"], headers={"User-Agent": USER_AGENT})
                with urllib.request.urlopen(req, timeout=10) as resp:
                    with open(local_path, "wb") as f:
                        f.write(resp.read())

            m["localPath"] = local_path
            downloaded.append(m)
        except Exception as e:
            # Still keep the meme even if local download failed
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
