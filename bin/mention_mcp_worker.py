#!/usr/bin/env python3
import asyncio
import json
import os
import sys
import time
import urllib.request
import urllib.parse
from playwright.async_api import async_playwright

SYNC_API_URL = "https://dashboard-blue-five-75.vercel.app/api/sync"
MENTIONS_API_URL = "https://dashboard-blue-five-75.vercel.app/api/mentions"
PITCH_API_KEY = "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB"
PITCH_MCP_URL = "https://api.trypitch.co/mcp"
ENV_PATH = "/home/adnan/x_bot/.env"

def load_env_token():
    if os.path.exists(ENV_PATH):
        with open(ENV_PATH) as f:
            for line in f:
                if line.startswith("X_USER_ACCESS_TOKEN="):
                    return line.split("=", 1)[1].strip().strip('\"\'')
    return None

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
        "Accept": "application/json, text/event-stream"
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
        print(f"[Worker MCP Error]: {e}")
    return None

def post_x_reply_official_api(tweet_id, text):
    """Primary: Official X API v2"""
    token = load_env_token()
    if not token:
        print("[Worker] No X_USER_ACCESS_TOKEN found in .env")
        return None

    url = "https://api.twitter.com/2/tweets"
    payload = {
        "text": text,
        "reply": {"in_reply_to_tweet_id": str(tweet_id)}
    }
    req = urllib.request.Request(url, data=json.dumps(payload).encode("utf-8"), headers={
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json"
    }, method="POST")

    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            reply_id = data.get("data", {}).get("id")
            if reply_id:
                print(f"[Official X API Success] Reply posted with ID {reply_id}")
                return reply_id
    except Exception as e:
        print(f"[Official X API Warning] Official API reply failed: {e}. Falling back to browser automation...")
    return None

async def post_x_reply_playwright_fallback(tweet_id, text, handle):
    """Secondary Fallback: Playwright Browser Automation"""
    state_file = "/home/adnan/x_bot/.browser-profile-trypitchdotco/storageState_trypitchdotco.json"
    clean_handle = handle.replace("@", "")
    target_tweet_url = f"https://x.com/{clean_handle}/status/{tweet_id}"

    try:
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            context = await browser.new_context(storage_state=state_file)
            page = await context.new_page()

            print(f"[Browser Fallback] Navigating to {target_tweet_url}...")
            await page.goto(target_tweet_url, wait_until="domcontentloaded")
            await page.wait_for_timeout(3000)

            reply_box = page.locator('div[data-testid="tweetTextarea_0"]').first
            if await reply_box.count() == 0:
                placeholder = page.locator('text="Post your reply"').first
                if await placeholder.count() > 0:
                    await placeholder.click()
                    await page.wait_for_timeout(1000)

            reply_box = page.locator('div[data-testid="tweetTextarea_0"]').first
            if await reply_box.count() > 0:
                await reply_box.fill(text)
                await page.wait_for_timeout(1000)
                reply_button = page.locator('button[data-testid="tweetButtonInline"]').first
                if await reply_button.count() > 0:
                    await reply_button.click()
                    await page.wait_for_timeout(3000)
                    print("[Browser Fallback Success] Reply posted via browser session")
                    await browser.close()
                    return "browser_delivered"

            await browser.close()
    except Exception as e:
        print(f"[Browser Fallback Error]: {e}")
    return None

def sync_job_to_dashboard(job):
    data = json.dumps({"mention_jobs": [job]}).encode("utf-8")
    req = urllib.request.Request(SYNC_API_URL, data=data, headers={"Content-Type": "application/json"}, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            pass
    except Exception as e:
        print(f"[Worker Sync Error]: {e}")

def fetch_mention_jobs():
    req = urllib.request.Request(MENTIONS_API_URL)
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            return data.get("jobs", [])
    except Exception as e:
        print(f"[Worker Fetch Error]: {e}")
        return []

async def run_mcp_queue_worker():
    print("=== STARTING UNIFIED WORKER DAEMON (PRIMARY: OFFICIAL X API | FALLBACK: PLAYWRIGHT) ===")
    
    while True:
        try:
            jobs = fetch_mention_jobs()
            
            for job in jobs:
                status = job.get("status")
                target_url = job.get("target_url")
                job_id = job.get("editor_job_id")
                user_handle = job.get("user_handle", "@user")
                raw_tweet_id = job.get("tweet_id", "").split("_")[0]

                # Direction 1: Supabase (pending) -> Pitch MCP (create_demo_video)
                if status == "pending" and target_url and target_url != "N/A":
                    print(f"[Worker] Claiming pending job for {user_handle} ({target_url})...")
                    mcp_res = call_pitch_mcp("create_demo_video", {
                        "url": target_url,
                        "instructions": f"Create a cinematic product demo video walkthrough for {target_url}. Highlight key features, value proposition, and user experience. Audio: Charon, Header: Light, Background: Ocean"
                    })
                    new_job_id = mcp_res.get("jobId") if mcp_res else None
                    if new_job_id:
                        job["editor_job_id"] = new_job_id
                        job["status"] = "rendering"
                        print(f"[Worker] Pitch MCP job created: {new_job_id}. Status set to rendering.")
                        sync_job_to_dashboard(job)

                # Direction 2: Pitch MCP (rendering) -> Poll status -> Post X Reply (Primary: API, Fallback: Browser) -> Supabase (delivered)
                elif status == "rendering" and job_id:
                    status_res = call_pitch_mcp("get_job", {"jobId": job_id})
                    if status_res:
                        mcp_status = status_res.get("status")
                        if mcp_status == "COMPLETED":
                            artifacts = status_res.get("artifacts", {})
                            s3_url = artifacts.get("final_with_cards") or artifacts.get("video") or status_res.get("s3_url") or f"https://trypitch.co/editor/{job_id}"
                            
                            print(f"[Worker] Job {job_id} COMPLETED! S3 Video URL: {s3_url}")
                            reply_text = f"Here is your product demo {user_handle} generated by @trypitchdotco! Hope you enjoy it: {s3_url} 🎬"
                            
                            # Primary Attempt: Official X API v2
                            x_reply_id = None
                            if raw_tweet_id and raw_tweet_id.isdigit():
                                x_reply_id = post_x_reply_official_api(raw_tweet_id, reply_text)
                            
                            # Secondary Fallback: Playwright Browser Automation
                            if not x_reply_id and raw_tweet_id and raw_tweet_id.isdigit():
                                x_reply_id = await post_x_reply_playwright_fallback(raw_tweet_id, reply_text, user_handle)
                            
                            job["status"] = "delivered"
                            job["s3_video_url"] = s3_url
                            if x_reply_id and x_reply_id != "browser_delivered":
                                job["x_reply_id"] = x_reply_id
                                
                            sync_job_to_dashboard(job)
                        elif mcp_status in ["FAILED", "ERROR"]:
                            print(f"[Worker] Job {job_id} FAILED on Pitch MCP")
                            job["status"] = "failed"
                            sync_job_to_dashboard(job)

        except Exception as e:
            print(f"[Worker Loop Error]: {e}")

        await asyncio.sleep(10)

if __name__ == "__main__":
    asyncio.run(run_mcp_queue_worker())
