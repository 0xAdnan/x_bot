#!/usr/bin/env python3
import asyncio
import json
import os
import re
import sys
import time
import urllib.request
import urllib.parse
from playwright.async_api import async_playwright

PITCH_API_KEY = "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB"
PITCH_MCP_URL = "https://api.trypitch.co/mcp"
SUPABASE_URL = "https://jwswpryozfxzaocimadp.supabase.co"
SUPABASE_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imp3c3dwcnlvemZ4emFvY2ltYWRwIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODYyNzQzMjQsImV4cCI6MjEwMTg1MDMyNH0.U575XIPsA12Y3JpZJ_gr9T7xH4WZafihkThQJvh8VNo"
PROCESSED_PH_FILE = "/home/adnan/x_bot/state/processed_ph_launches.json"

os.makedirs("/home/adnan/x_bot/state", exist_ok=True)

PH_QUERIES = [
    'ProductHunt',
    '@ProductHunt',
    '"launched on Product Hunt"',
    '"live on Product Hunt"'
]

def load_processed_ph():
    if os.path.exists(PROCESSED_PH_FILE):
        try:
            with open(PROCESSED_PH_FILE) as f:
                return set(json.load(f))
        except Exception:
            pass
    return set()

def save_processed_ph(key):
    processed = load_processed_ph()
    processed.add(key)
    with open(PROCESSED_PH_FILE, "w") as f:
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

def post_x_api_reply(tweet_id, text):
    x_token = "ZDdibjRaZHJ5Y1BQbDM2ei1jM01tOVZoazR1VVdMVDR4eTVkdjRHRGs0V3lxOjE3ODYzMDY3MzA4Nzg6MTowOmF0OjE"
    url = "https://api.twitter.com/2/tweets"
    payload = {
        "text": text,
        "reply": {"in_reply_to_tweet_id": tweet_id}
    }
    req = urllib.request.Request(url, data=json.dumps(payload).encode("utf-8"), headers={
        "Authorization": f"Bearer {x_token}",
        "Content-Type": "application/json"
    }, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            print(f"[ProductHunt X Reply] Status: {resp.status}")
            return True
    except Exception as e:
        print(f"[ProductHunt X Reply Error]: {e}")
        return False

def sync_supabase_ph_job(tweet_id, handle, target_url, job_id, status, video_url):
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
            print(f"[Supabase Sync PH Job] Synced ({status})")
    except Exception as e:
        print(f"[Supabase Sync Error]: {e}")

async def monitor_product_hunt_launches():
    state_file = "/home/adnan/x_bot/.browser-profile-trypitchdotco/storageState_trypitchdotco.json"
    print("=== STARTING PRODUCT HUNT AUTO-DEMO LAUNCH MONITOR ===")
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(storage_state=state_file)
        page = await context.new_page()
        
        query_idx = 0
        
        while True:
            query = PH_QUERIES[query_idx % len(PH_QUERIES)]
            query_idx += 1
            
            print(f"\n[Product Hunt Monitoring Query]: '{query}'...")
            
            try:
                search_url = f"https://x.com/search?q={urllib.parse.quote(query)}&f=live"
                await page.goto(search_url, wait_until="domcontentloaded")
                await page.wait_for_timeout(3000)
                
                tweets = await page.locator('article[data-testid="tweet"]').all()
                processed = load_processed_ph()
                
                for tweet_el in tweets[:5]:
                    text = await tweet_el.inner_text()
                    lines = text.split("\n")
                    
                    user_handle = None
                    for l in lines:
                        if l.startswith("@") and "producthunt" not in l.lower() and "trypitchdotco" not in l.lower():
                            user_handle = l.strip()
                            break
                            
                    if not user_handle:
                        continue
                        
                    ph_key = f"{user_handle}_{hash(text)}"
                    if ph_key in processed:
                        continue
                        
                    # Extract product URL
                    url_match = re.search(r'https?://[^\s]+', text) or re.search(r'([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:\/[^\s]*)?)', text)
                    if not url_match:
                        save_processed_ph(ph_key)
                        continue
                        
                    target_url = url_match.group(0)
                    if not target_url.startswith("http"):
                        target_url = f"https://{target_url}"
                        
                    if "producthunt.com" in target_url or "s3.trypitch.co" in target_url:
                        save_processed_ph(ph_key)
                        continue
                        
                    print(f"🔥 [NEW PRODUCT HUNT LAUNCH DETECTED]: {user_handle} launching {target_url}!")
                    save_processed_ph(ph_key)
                    
                    # STEP A: Send Instant Congratulations & Offer Reply
                    ack_text = f"Congrats on the launch {user_handle}! 🚀 Generating a 60s AI video demo for {target_url} using PITCH now, we'll reply right here as soon as it's ready!"
                    print(f"Sending PH Receipt Reply to {user_handle}...")
                    
                    # STEP B: Trigger Pitch MCP Video Creation
                    print(f"Triggering Pitch MCP launch demo creation for {target_url}...")
                    mcp_res = call_pitch_mcp("create_demo_video", {
                        "url": target_url,
                        "instructions": f"Create a cinematic Product Hunt launch demo walkthrough of {target_url}. Highlight key features, value proposition, and user experience.",
                        "voice": "Charon",
                        "subtitles": False,
                        "theme": "light",
                        "background": "ocean",
                        "shape": "rounded",
                        "inset": "0.75",
                        "browserHeader": "light"
                    })
                    
                    job_id = mcp_res.get("jobId") if mcp_res else None
                    if job_id:
                        print(f"Pitch MCP Launch Job Created! Job ID: {job_id}")
                        sync_supabase_ph_job(ph_key, user_handle, target_url, job_id, "rendering", "")
                    
            except Exception as e:
                print(f"[Product Hunt Loop Exception]: {e}")
                
            # Sleep 30s between PH scans
            await asyncio.sleep(30)

if __name__ == "__main__":
    asyncio.run(monitor_product_hunt_launches())
