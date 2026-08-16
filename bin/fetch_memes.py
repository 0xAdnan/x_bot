#!/usr/bin/env python3
"""
fetch_memes.py — Multi-Source Autonomous Meme Scraper & Sarcastic Rage-Bait Engine.
Scrapes real, high-heat tech/developer memes from:
  1. X / Tech Twitter (via Playwright search min_faves:50 filter:media)
  2. Reddit (r/ProgrammerHumor, r/ChatGPTMemes, r/softwaregore)
  3. 4chan /g/ (Technology)
Generates punchy, ultra-sarcastic developer & founder commentary (ZERO robotic AI em-dashes).
"""

import os
import sys
import json
import time
import urllib.request
import re
import asyncio
from playwright.async_api import async_playwright

REPO_DIR = "/home/adnan/x_bot"
MEME_DIR = os.path.join(REPO_DIR, "assets", "memes")
CACHE_FILE = os.path.join(REPO_DIR, "data", "fetched_memes.json")
STORAGE_STATE = os.path.join(REPO_DIR, ".browser-profile-adnanspitch", "storageState.json")
if not os.path.exists(STORAGE_STATE):
    STORAGE_STATE = os.path.join(REPO_DIR, ".browser-profile", "storageState.json")

os.makedirs(MEME_DIR, exist_ok=True)
os.makedirs(os.path.dirname(CACHE_FILE), exist_ok=True)

USER_AGENT = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"

def cleanup_old_memes(max_age_hours=24):
    """Delete downloaded meme images older than max_age_hours (default 24h) to avoid disk bloat."""
    if not os.path.exists(MEME_DIR):
        return
    now = time.time()
    cutoff = now - (max_age_hours * 3600)
    deleted_count = 0
    for filename in os.listdir(MEME_DIR):
        file_path = os.path.join(MEME_DIR, filename)
        if os.path.isfile(file_path):
            try:
                mtime = os.path.getmtime(file_path)
                if mtime < cutoff:
                    os.remove(file_path)
                    deleted_count += 1
            except Exception:
                pass
    if deleted_count > 0:
        print(f"[Cleanup] Deleted {deleted_count} meme images older than {max_age_hours} hours from assets/memes/")

def generate_sarcastic_ragebait_caption(title, source="reddit", tweet_text=""):
    raw = (tweet_text if tweet_text and len(tweet_text) > len(title) else title).lower()
    
    # Clean up formatting
    s = re.sub(r'([a-z])([A-Z])', r'\1 \2', raw)
    s = s.replace('_', ' ').replace('-', ' ').replace('—', ' ').replace('–', ' ').strip()
    clean = re.sub(r'\s+', ' ', s).replace('&#039;', "'").replace('&quot;', '"').replace('&amp;', '&')

    # Keyword based ultra-sarcastic developer & founder rage-bait takes
    if any(k in clean for k in ["thanks", "hate", "terrible", "awful", "cursed", "weird"]):
        takes = [
            "thanks i hate it. this is why aliens refuse to visit earth",
            "every day we stray further from god's light and closer to production outages",
            "whoever approved this pr needs their git credentials revoked immediately"
        ]
    elif any(k in clean for k in ["first project", "nervous", "junior", "student", "beginner"]):
        takes = [
            "me pushing an untested express server to production with hardcoded api keys in the repo",
            "bro is raising a $3M seed round with 1 supabase table, a tailwind template, and a dream",
            "first project energy: 0 error handling, 14 console.logs, 100% confidence"
        ]
    elif any(k in clean for k in ["type", "typescript", "typeslop", "any", "javascript"]):
        takes = [
            "typescript devs spending 3 hours designing 400 lines of generic types just to cast it as 'any' at the end",
            "typeslop in full production effect. if it compiles, it ships",
            "adding ': any' to every single variable until the red squiggly lines disappear"
        ]
    elif any(k in clean for k in ["ai", "claude", "cursor", "vibe", "model", "chatgpt", "llm", "prompt"]):
        takes = [
            "vibe coding has turned software engineering into high-stakes prompt poetry. 1 typo and your db schema is in the shadow realm",
            "spending 10 mins vibe-coding an entire app with claude then spending 4 days sweating through 50 demo video retakes",
            "90% of prompt engineering is just typing 'are you sure' until the model gaslights itself into changing the answer",
            "me explaining to investors that our proprietary neural agent is doing deep multi-step reasoning (it is literally a while loop with an api key)"
        ]
    elif any(k in clean for k in ["friday", "weekend", "deploy", "merge", "commit", "push", "main", "git"]):
        takes = [
            "pushing directly to main at 4:59 pm on a friday and immediately closing the laptop",
            "the junior dev submitting an 8,000-line claude hallucination at 4:58 pm and logging off slack",
            "bro has 0 public commits and just deployed a $40k mrr app to production"
        ]
    elif any(k in clean for k in ["screen", "demo", "loom", "video", "record", "recording"]):
        takes = [
            "spending 4 hours doing 50 retakes on screen studio because your voice cracked at 0:58 is psychotic founder behavior",
            "the 4 stages of recording a demo: 1. this takes 5 mins 2. mic was muted 3. notification popup 4. it is 4 am and you're learning after effects",
            "polymarket 95% odds that devs would rather rewrite their entire backend in rust than record a 2-minute product demo manually"
        ]
    elif any(k in clean for k in ["phone", "apple", "samsung", "hardware", "camera", "nokia"]):
        takes = [
            "paying $1,800 for a phone just to check stripe dashboard and argue with strangers on tech twitter",
            "the tech industry running out of ideas and adding 6 more camera lenses to look at terminal errors in 8k"
        ]
    elif any(k in clean for k in ["gore", "error", "bug", "crash", "bios", "boot"]):
        takes = [
            "my production backend held together by 3 edge functions, zero error handling, and sheer vibes",
            "when the senior architect said 'this edge case will never happen in production'",
            "average tech stack in 2026: 15 ai wrappers, 1 supabase instance, and prayer"
        ]
    else:
        # Default pure tech sarcasm
        takes = [
            clean if len(clean) < 70 else clean[:67] + "...",
            "nothing tests founder sanity like debugging a race condition that turned out to be an env var typo",
            "spending $20M training a frontier model just for people to use it to write cover letters and regex",
            "the best code is no code. the second best code is code generated by claude that you pretend you understand in the pr review"
        ]

    import random
    return random.choice(takes)

def fetch_reddit_memes(subreddits=["ProgrammerHumor", "ChatGPTMemes", "softwaregore"], limit=4):
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
                    caption = generate_sarcastic_ragebait_caption(title, "reddit")
                    memes.append({
                        "id": f"reddit_{m.get('postLink', '').split('/')[-1]}",
                        "title": title,
                        "source": f"r/{sub}",
                        "sourceUrl": m.get("postLink", ""),
                        "imageUrl": img_url,
                        "ups": m.get("ups", 0),
                        "caption": caption,
                        "format": "reddit_meme"
                    })
        except Exception as e:
            print(f"[Warn] Reddit fetch failed for {sub}: {e}")
    return memes

def fetch_4chan_memes(board="g", limit=4):
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

                    if any(k in thread_text for k in keywords) or (sub and len(sub) > 4):
                        img_url = f"https://i.4cdn.org/{board}/{tim}{ext}"
                        title = sub if sub else (clean_com[:60] if clean_com else f"4chan /{board}/ Discussion")
                        
                        caption = generate_sarcastic_ragebait_caption(title, "4chan", thread_text)
                        memes.append({
                            "id": f"4chan_{thread.get('no')}",
                            "title": title,
                            "source": f"4chan /{board}/",
                            "sourceUrl": f"https://boards.4chan.org/{board}/thread/{thread.get('no')}",
                            "imageUrl": img_url,
                            "ups": thread.get("replies", 0),
                            "caption": caption,
                            "format": "4chan_meme"
                        })
                        if len(memes) >= limit:
                            return memes
    except Exception as e:
        print(f"[Warn] 4chan fetch failed for /{board}/: {e}")
    return memes

async def fetch_x_tech_memes(limit=5):
    """
    Scrapes live viral tech & developer memes directly from X / Tech Twitter.
    """
    memes = []
    print("[Meme Scraper] Scanning X / Tech Twitter for viral media memes...")
    try:
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            context = await browser.new_context(
                storage_state=STORAGE_STATE if os.path.exists(STORAGE_STATE) else None,
                user_agent=USER_AGENT
            )
            page = await context.new_page()
            
            queries = [
                '("programmer humor" OR "coding meme" OR "developer meme" OR "vibe coding") min_faves:50 filter:media',
                '(from:PR0GRAMMERHUM0R OR from:shitpostgate OR from:IamaDev) filter:media'
            ]

            for q in queries:
                url = f"https://x.com/search?q={urllib.request.quote(q)}&f=live"
                await page.goto(url, wait_until="domcontentloaded")
                await page.wait_for_timeout(3500)

                articles = page.locator('article[data-testid="tweet"]')
                cnt = await articles.count()

                for i in range(min(cnt, 4)):
                    art = articles.nth(i)
                    text_el = art.locator('div[data-testid="tweetText"]').first
                    img_el = art.locator('div[data-testid="tweetPhoto"] img').first
                    link_el = art.locator('a[href*="/status/"]').first

                    if await img_el.count() > 0 and await link_el.count() > 0:
                        img_src = await img_el.get_attribute('src')
                        href = await link_el.get_attribute('href')
                        text = await text_el.inner_text() if await text_el.count() > 0 else ""
                        
                        if img_src and "twimg.com/media" in img_src:
                            # Upgrade to high-res name=large
                            hi_res = re.sub(r'name=[a-z0-9]+', 'name=large', img_src)
                            tweet_id = href.split('/status/')[-1].split('?')[0] if href else str(i)
                            tweet_url = f"https://x.com{href}" if href.startswith('/') else href

                            caption = generate_sarcastic_ragebait_caption(text[:60], "x_twitter", text)

                            memes.append({
                                "id": f"x_{tweet_id}",
                                "title": text[:60] if text else "Tech Twitter Viral Meme",
                                "source": "X / Tech Twitter",
                                "sourceUrl": tweet_url,
                                "imageUrl": hi_res,
                                "ups": 150,
                                "caption": caption,
                                "format": "x_meme"
                            })
                            if len(memes) >= limit:
                                break

                if len(memes) >= limit:
                    break

            await browser.close()
    except Exception as e:
        print(f"[Warn] X meme scraper failed: {e}")
    return memes

def download_meme_images(memes):
    downloaded = []
    for m in memes:
        try:
            ext = "png"
            if ".jpg" in m["imageUrl"] or "format=jpg" in m["imageUrl"]:
                ext = "jpg"
            elif ".webp" in m["imageUrl"]:
                ext = "webp"

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
            downloaded.append(m)
    return downloaded

async def main():
    print("=== [MULTI-SOURCE VIRAL TECH MEME SCRAPER] ===")
    
    # 0. Clean up images older than 24 hours
    cleanup_old_memes(max_age_hours=24)
    
    # 1. Fetch from X / Tech Twitter
    x_memes = await fetch_x_tech_memes(limit=4)

    # 2. Fetch from Reddit
    reddit_memes = fetch_reddit_memes(limit=4)

    # 3. Fetch from 4chan
    chan_memes = fetch_4chan_memes(board="g", limit=4)

    all_memes = x_memes + reddit_memes + chan_memes
    processed = download_meme_images(all_memes)

    with open(CACHE_FILE, "w") as f:
        json.dump(processed, f, indent=2)

    print(f"[OK] Fetched & cached {len(processed)} viral tech memes from X, Reddit, and 4chan!")
    for m in processed:
        print(f"  • [{m['source']}] {m['title'][:40]} | Caption: \"{m['caption'][:60]}...\"")

if __name__ == "__main__":
    asyncio.run(main())
