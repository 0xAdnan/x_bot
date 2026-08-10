#!/usr/bin/env python3
import asyncio
import json
import os
import sys
import time
import urllib.request
import urllib.parse

SYNC_API_URL = "https://dashboard-blue-five-75.vercel.app/api/sync"
MENTIONS_API_URL = "https://dashboard-blue-five-75.vercel.app/api/mentions"
PITCH_API_KEY = "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB"
PITCH_MCP_URL = "https://api.trypitch.co/mcp"
X_USER_TOKEN = "ZDdibjRaZHJ5Y1BQbDM2ei1jM01tOVZoazR1VVdMVDR4eTVkdjRHRGs0V3lxOjE3ODYzMDY3MzA4Nzg6MTowOmF0OjE"

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

def post_x_reply(tweet_id, text):
    url = "https://api.twitter.com/2/tweets"
    payload = {
        "text": text,
        "reply": {"in_reply_to_tweet_id": tweet_id}
    }
    req = urllib.request.Request(url, data=json.dumps(payload).encode("utf-8"), headers={
        "Authorization": f"Bearer {X_USER_TOKEN}",
        "Content-Type": "application/json"
    }, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            return data.get("data", {}).get("id")
    except Exception as e:
        print(f"[Worker X Reply Error]: {e}")
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
    print("=== STARTING UNIFIED BIDIRECTIONAL PITCH MCP WORKER DAEMON ===")
    
    while True:
        try:
            jobs = fetch_mention_jobs()
            
            for job in jobs:
                status = job.get("status")
                target_url = job.get("target_url")
                job_id = job.get("editor_job_id")
                user_handle = job.get("user_handle")
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

                # Direction 2: Pitch MCP (rendering) -> Poll status -> Post X Reply -> Supabase (delivered)
                elif status == "rendering" and job_id:
                    status_res = call_pitch_mcp("get_job", {"jobId": job_id})
                    if status_res:
                        mcp_status = status_res.get("status")
                        if mcp_status == "COMPLETED":
                            # Extract S3 video URL
                            artifacts = status_res.get("artifacts", {})
                            s3_url = artifacts.get("final_with_cards") or artifacts.get("video") or status_res.get("s3_url") or f"https://trypitch.co/editor/{job_id}"
                            
                            print(f"[Worker] Job {job_id} COMPLETED! S3 Video URL: {s3_url}")
                            
                            # Post X Reply
                            reply_text = f"Here is your product demo {user_handle} generated by @trypitchdotco! Hope you enjoy it: {s3_url} 🎬"
                            x_reply_id = None
                            if raw_tweet_id and raw_tweet_id.isdigit():
                                x_reply_id = post_x_reply(raw_tweet_id, reply_text)
                            
                            job["status"] = "delivered"
                            job["s3_video_url"] = s3_url
                            if x_reply_id:
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
