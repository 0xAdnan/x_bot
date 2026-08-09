#!/usr/bin/env python3
import asyncio
import json
import os
import re
import sys
import time
import urllib.request
from playwright.async_api import async_playwright

PITCH_API_KEY = "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB"
PITCH_MCP_URL = "https://api.trypitch.co/mcp"
SUPABASE_URL = "https://jwswpryozfxzaocimadp.supabase.co"
SUPABASE_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imp3c3dwcnlvemZ4emFvY2ltYWRwIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODYyNzQzMjQsImV4cCI6MjEwMTg1MDMyNH0.U575XIPsA12Y3JpZJ_gr9T7xH4WZafihkThQJvh8VNo"
PROCESSED_TWEETS_FILE = "/home/adnan/x_bot/state/processed_tweets.json"

os.makedirs("/home/adnan/x_bot/state", exist_ok=True)

def load_processed_tweets():
    if os.path.exists(PROCESSED_TWEETS_FILE):
        try:
            with open(PROCESSED_TWEETS_FILE) as f:
                return set(json.load(f))
        except Exception:
            pass
    return set()

def save_processed_tweet(tweet_id):
    processed = load_processed_tweets()
    processed.add(str(tweet_id))
    with open(PROCESSED_TWEETS_FILE, "w") as f:
        json.dump(list(processed), f)

def call_pitch_mcp(tool_name, arguments):
    payload = {
        "jsonrpc": "2.0",
        "id": int(time.time()),
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments}
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(PITCH_MCP_URL, data=data, headers={
        "Authorization": f"Bearer {PITCH_API_KEY}",
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
        "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36"
    })
    try:
        with urllib.request.urlopen(req) as resp:
            text = resp.read().decode("utf-8")
            for line in text.split("\n"):
                if line.startswith("data: "):
                    parsed = json.loads(line[6:])
                    content = parsed.get("result", {}).get("content", [])
                    if content and content[0].get("type") == "text":
                        return json.loads(content[0].get("text"))
    except Exception as e:
        print(f"[Pitch MCP Error]: {e}")
    return None

def sync_supabase_job(tweet_id, handle, target_url, job_id, status, video_url):
    url = f"{SUPABASE_URL}/rest/v1/mention_jobs"
    payload = [{
        "tweet_id": str(tweet_id),
        "user_handle": handle,
        "target_url": target_url,
        "editor_job_id": job_id,
        "status": status,
        "s3_video_url": video_url
    }]
    req = urllib.request.Request(url, data=json.dumps(payload).encode("utf-8"), headers={
        "apikey": SUPABASE_KEY,
        "Authorization": f"Bearer {SUPABASE_KEY}",
        "Content-Type": "application/json",
        "Prefer": "resolution=merge-duplicates"
    }, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            print(f"[Supabase Sync] Job synced ({status})")
    except Exception as e:
        print(f"[Supabase Sync Error]: {e}")

async def poll_mention_feed():
    state_file = "/home/adnan/x_bot/.browser-profile-trypitchdotco/storageState_trypitchdotco.json"
    print("=== STARTING 10-SECOND INSTANT FALLBACK MENTION DAEMON ===")
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(storage_state=state_file)
        page = await context.new_page()
        
        while True:
            try:
                # Scan live search every 10s
                await page.goto("https://x.com/search?q=%40trypitchdotco&f=live", wait_until="domcontentloaded")
                await page.wait_for_timeout(3000)
                
                tweets = await page.locator('article[data-testid="tweet"]').all()
                processed = load_processed_tweets()
                
                for tweet_el in tweets[:5]:
                    text = await tweet_el.inner_text()
                    lines = text.split("\n")
                    
                    # Extract handle
                    user_handle = "@user"
                    for l in lines:
                        if l.startswith("@") and "trypitchdotco" not in l.lower():
                            user_handle = l.strip()
                            break
                            
                    tweet_key = f"{user_handle}_{hash(text)}"
                    if tweet_key in processed:
                        continue
                        
                    # Filter out tweets from @trypitchdotco itself or bot replies
                    if "trypitchdotco" in user_handle.lower() or "Here is your product demo" in text:
                        save_processed_tweet(tweet_key)
                        continue
                        
                    # Extract URL or domain name
                    url_match = re.search(r'https?://[^\s]+', text) or re.search(r'([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:\/[^\s]*)?)', text)
                    if not url_match:
                        save_processed_tweet(tweet_key)
                        continue
                        
                    target_url = url_match.group(0)
                    if not target_url.startswith("http://") and not target_url.startswith("https://"):
                        target_url = f"https://{target_url}"
                        
                    # Ignore pitch s3 links
                    if "s3.trypitch.co" in target_url:
                        save_processed_tweet(tweet_key)
                        continue
                        
                    print(f"\n[FALLBACK DAEMON DETECTED MENTION]: {text[:100]}... (from {user_handle})")
                    save_processed_tweet(tweet_key)
                    
                    # Trigger Pitch MCP Video Job
                    print(f"Triggering Pitch MCP video creation for {target_url}...")
                    mcp_res = call_pitch_mcp("create_demo_video", {
                        "url": target_url,
                        "instructions": f"Create a cinematic product demo walkthrough of {target_url}. Audio: Charon, Header: Light, Background: Ocean"
                    })
                    
                    job_id = mcp_res.get("jobId") if mcp_res else None
                    if job_id:
                        print(f"Pitch MCP Job Created! Job ID: {job_id}")
                        sync_supabase_job(tweet_key, user_handle, target_url, job_id, "rendering", "")
                    
            except Exception as e:
                print(f"[Daemon Loop Error]: {e}")
                
            # Sleep 10 seconds before next scan
            await asyncio.sleep(10)

if __name__ == "__main__":
    asyncio.run(poll_mention_feed())
