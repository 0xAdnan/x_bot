#!/usr/bin/env python3
import asyncio
import json
import os
import re
import sys
import time
import urllib.request
import urllib.parse
import hashlib
from playwright.async_api import async_playwright

PITCH_API_KEY = "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB"
PITCH_MCP_URL = "https://api.trypitch.co/mcp"
SUPABASE_URL = "https://jwswpryozfxzaocimadp.supabase.co"
SYNC_API_URL = "https://dashboard-blue-five-75.vercel.app/api/sync"
PROCESSED_TWEETS_FILE = "/home/adnan/x_bot/state/processed_tweets.json"

os.makedirs("/home/adnan/x_bot/state", exist_ok=True)

def ping_heartbeat():
    """Send live heartbeat pulse to Supabase via Sync API."""
    data = json.dumps({"activities": [{
        "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "action": "heartbeat",
        "handle": "@trypitchdotco",
        "segment": "mention_daemon",
        "variant": "",
        "detail": "Heartbeat pulse from mention_daemon.py",
        "result": "ok"
    }]}).encode("utf-8")
    req = urllib.request.Request(SYNC_API_URL, data=data, headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            pass
    except Exception:
        pass

def get_supabase_key():
    for key in ["SUPABASE_SERVICE_ROLE_KEY", "SUPABASE_ANON_KEY", "SUPABASE_KEY"]:
        val = os.environ.get(key)
        if val and not val.endswith("...8VNo"):
            return val
    for env_file in [
        "/home/adnan/x_bot/dashboard/.env.production.local",
        "/home/adnan/x_bot/dashboard/.env.local",
        "/home/adnan/x_bot/.env"
    ]:
        if os.path.exists(env_file):
            try:
                with open(env_file) as f:
                    for line in f:
                        if "SUPABASE_ANON_KEY=" in line or "SUPABASE_SERVICE_ROLE_KEY=" in line:
                            val = line.split("=", 1)[1].strip().strip('\"\'')
                            if val and not val.endswith("...8VNo"):
                                return val
            except Exception:
                pass
    return "eyJhbG...8VNo"

SUPABASE_KEY = get_supabase_key()

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

def check_supabase_job_exists(tweet_id, user_handle, target_url):
    if not tweet_id:
        return False, None
    key = get_supabase_key()
    url = f"{SUPABASE_URL}/rest/v1/mention_jobs?or=(tweet_id.eq.{urllib.parse.quote(str(tweet_id))},and(user_handle.eq.{urllib.parse.quote(user_handle)},target_url.eq.{urllib.parse.quote(target_url)}))&select=id,status,editor_job_id"
    req = urllib.request.Request(url, headers={
        "apikey": key,
        "Authorization": f"Bearer {key}"
    })
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            if data and len(data) > 0:
                return True, data[0]
    except Exception as e:
        print(f"[Supabase Check Error]: {e}")
    return False, None

def sync_supabase_job(tweet_id, handle, target_url, job_id, status, video_url):
    key = get_supabase_key()
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
        "apikey": key,
        "Authorization": f"Bearer {key}",
        "Content-Type": "application/json",
        "Prefer": "resolution=merge-duplicates"
    }, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            print(f"[Supabase Sync] Job synced ({status}) for Tweet ID {tweet_id}")
    except Exception as e:
        print(f"[Supabase Sync Error]: {e}")

async def process_tweet_element(tweet_el):
    text = await tweet_el.inner_text()
    lines = text.split("\n")
    
    user_handle = "@user"
    for l in lines:
        if l.startswith("@") and "trypitchdotco" not in l.lower():
            user_handle = l.strip()
            break

    tweet_id = None
    tweet_url = None
    try:
        status_link_el = tweet_el.locator('a[href*="/status/"]').first
        if await status_link_el.count() > 0:
            href = await status_link_el.get_attribute("href")
            if href and "/status/" in href:
                parts = href.split("/status/")
                tweet_id = parts[1].split("?")[0].split("/")[0]
                clean_user = parts[0].replace("/", "")
                if clean_user:
                    user_handle = f"@{clean_user}"
                tweet_url = f"https://x.com/{clean_user}/status/{tweet_id}"
    except Exception:
        pass

    if not tweet_id:
        tweet_id = hashlib.sha256(f"{user_handle}_{text[:100]}".encode('utf-8')).hexdigest()[:18]
        clean_user = user_handle.replace("@", "")
        tweet_url = f"https://x.com/{clean_user}"

    processed = load_processed_tweets()
    if tweet_id in processed or f"{user_handle}_{tweet_id}" in processed:
        return

    if "trypitchdotco" in user_handle.lower() or "Here is your product demo" in text:
        save_processed_tweet(tweet_id)
        return

    url_match = re.search(r'https?://[^\s]+', text) or re.search(r'([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:\/[^\s]*)?)', text)
    target_url = url_match.group(0) if url_match else "N/A"
    
    if target_url != "N/A":
        if not target_url.startswith("http://") and not target_url.startswith("https://"):
            target_url = f"https://{target_url}"
        if "s3.trypitch.co" in target_url:
            save_processed_tweet(tweet_id)
            return

    exists, existing_job = check_supabase_job_exists(tweet_id, user_handle, target_url)
    if exists:
        save_processed_tweet(tweet_id)
        return

    clean_text = text.replace('\n', ' ')
    print("\n" + "="*70)
    print("[MENTION BOT DETECTED NEW TWEET]")
    print(f"Requested By: {user_handle}")
    print(f"Mention Tweet URL: {tweet_url}")
    print(f"Tweet Text: {clean_text}")
    print(f"Target Product URL: {target_url}")
    print("="*70 + "\n")

    save_processed_tweet(tweet_id)

    if target_url == "N/A":
        print(f"[Mention Log] Tracking mention from {user_handle} (No direct URL found in text)")
        sync_supabase_job(tweet_id, user_handle, "N/A", "", "no_url_found", "")
        return

    print(f"Triggering Pitch MCP video creation for {target_url}...")
    mcp_res = call_pitch_mcp("create_demo_video", {
        "url": target_url,
        "instructions": f"Create a cinematic product demo walkthrough of {target_url}. Audio: Charon, Header: Light, Background: Ocean"
    })

    job_id = mcp_res.get("jobId") if mcp_res else None
    if job_id:
        print(f"Pitch MCP Job Created! Job ID: {job_id}")
        sync_supabase_job(tweet_id, user_handle, target_url, job_id, "rendering", "")
    else:
        print("[Pitch MCP Warning] Failed to get jobId from response, logging as submitted")
        sync_supabase_job(tweet_id, user_handle, target_url, "", "submitted", "")

async def poll_mention_feed():
    state_file = "/home/adnan/x_bot/.browser-profile-trypitchdotco/storageState_trypitchdotco.json"
    print("=== STARTING REAL-TIME DUAL-SCAN MENTION DAEMON (NOTIFICATIONS + SEARCH) ===")
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(storage_state=state_file)
        page = await context.new_page()
        
        urls_to_scan = [
            "https://x.com/notifications",
            "https://x.com/search?q=%40trypitchdotco&f=live"
        ]
        
        while True:
            # Send live heartbeat pulse
            ping_heartbeat()

            for scan_url in urls_to_scan:
                try:
                    await page.goto(scan_url, wait_until="domcontentloaded")
                    await page.wait_for_timeout(3000)
                    
                    tweets = await page.locator('article[data-testid="tweet"]').all()
                    
                    for tweet_el in tweets[:20]:
                        await process_tweet_element(tweet_el)
                        
                except Exception as e:
                    print(f"[Daemon Scan Error on {scan_url}]: {e}")

            await asyncio.sleep(10)

if __name__ == "__main__":
    asyncio.run(poll_mention_feed())
