#!/usr/bin/env python3
import json
import os
import urllib.request

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STATE_DIR = os.path.join(BASE_DIR, "state")
SYNC_API_URL = os.environ.get("SYNC_API_URL", "https://dashboard-blue-five-75.vercel.app/api/sync")

def read_jsonl(filepath):
    items = []
    if os.path.exists(filepath):
        with open(filepath, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        items.append(json.loads(line))
                    except json.JSONDecodeError:
                        pass
    return items

def sync_all():
    activities_file = os.path.join(STATE_DIR, "activity-log.jsonl")
    prospects_file = os.path.join(STATE_DIR, "prospects.jsonl")

    activities = read_jsonl(activities_file)
    prospects = read_jsonl(prospects_file)

    if not activities and not prospects:
        print("[Sync] No activities or prospects to sync.")
        return

    payload = {
        "activities": activities,
        "prospects": prospects
    }

    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(SYNC_API_URL, data=data, headers={
        "Content-Type": "application/json"
    }, method="POST")

    try:
        with urllib.request.urlopen(req) as resp:
            res_text = resp.read().decode("utf-8")
            print(f"[Sync Success]: {res_text}")
    except Exception as e:
        print(f"[Sync Error]: {e}")

if __name__ == "__main__":
    print("Starting Centralized Deduplicated Supabase Sync...")
    sync_all()
