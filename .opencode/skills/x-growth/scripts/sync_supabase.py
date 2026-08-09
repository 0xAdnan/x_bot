#!/usr/bin/env python3
import json
import os
import urllib.request

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STATE_DIR = os.path.join(BASE_DIR, "state")

SUPABASE_URL = os.environ.get("SUPABASE_URL", "https://jwswpryozfxzaocimadp.supabase.co")
SUPABASE_KEY = os.environ.get("SUPABASE_ANON_KEY", "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imp3c3dwcnlvemZ4emFvY2ltYWRwIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODYyNzQzMjQsImV4cCI6MjEwMTg1MDMyNH0.U575XIPsA12Y3JpZJ_gr9T7xH4WZafihkThQJvh8VNo")

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

def get_existing_activities():
    url = f"{SUPABASE_URL}/rest/v1/activities?select=ts,action,detail"
    req = urllib.request.Request(url, headers={
        "apikey": SUPABASE_KEY,
        "Authorization": f"Bearer {SUPABASE_KEY}"
    })
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode())
            return {(x.get("ts"), x.get("action"), x.get("detail")) for x in data}
    except Exception as e:
        print(f"[Sync] Fetch existing activities failed: {e}")
        return set()

def sync_activities():
    if not SUPABASE_URL or not SUPABASE_KEY:
        print("[Sync] Missing SUPABASE_URL or SUPABASE_KEY environment variables.")
        return

    filepath = os.path.join(STATE_DIR, "activity-log.jsonl")
    activities = read_jsonl(filepath)
    if not activities:
        print("[Sync] No activities to sync.")
        return

    existing = get_existing_activities()
    new_activities = [a for a in activities if (a.get("ts"), a.get("action"), a.get("detail")) not in existing]

    if not new_activities:
        print("[Sync] Activities up to date. No new entries to sync.")
        return

    url = f"{SUPABASE_URL}/rest/v1/activities"
    headers = {
        "apikey": SUPABASE_KEY,
        "Authorization": f"Bearer {SUPABASE_KEY}",
        "Content-Type": "application/json"
    }

    req = urllib.request.Request(url, data=json.dumps(new_activities).encode("utf-8"), headers=headers, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            print(f"[Sync] Synced {len(new_activities)} new activity entries. Status: {resp.status}")
    except Exception as e:
        print(f"[Sync] Activities sync failed: {e}")

def sync_prospects():
    if not SUPABASE_URL or not SUPABASE_KEY:
        return

    filepath = os.path.join(STATE_DIR, "prospects.jsonl")
    prospects = read_jsonl(filepath)
    if not prospects:
        print("[Sync] No prospects to sync.")
        return

    url = f"{SUPABASE_URL}/rest/v1/prospects"
    headers = {
        "apikey": SUPABASE_KEY,
        "Authorization": f"Bearer {SUPABASE_KEY}",
        "Content-Type": "application/json",
        "Prefer": "resolution=merge-duplicates"
    }

    req = urllib.request.Request(url, data=json.dumps(prospects).encode("utf-8"), headers=headers, method="POST")
    try:
        with urllib.request.urlopen(req) as resp:
            print(f"[Sync] Prospects sync status: {resp.status}")
    except Exception as e:
        print(f"[Sync] Prospects sync failed: {e}")

def sync_insights():
    if not SUPABASE_URL or not SUPABASE_KEY:
        return

    filepath = os.path.join(STATE_DIR, "insights.md")
    if os.path.exists(filepath):
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()

        url = f"{SUPABASE_URL}/rest/v1/insights"
        headers = {
            "apikey": SUPABASE_KEY,
            "Authorization": f"Bearer {SUPABASE_KEY}",
            "Content-Type": "application/json",
            "Prefer": "resolution=merge-duplicates"
        }
        payload = json.dumps([{"id": 1, "content": content}]).encode("utf-8")
        req = urllib.request.Request(url, data=payload, headers=headers, method="POST")
        try:
            with urllib.request.urlopen(req) as resp:
                print(f"[Sync] Insights sync status: {resp.status}")
        except Exception as e:
            print(f"[Sync] Insights sync failed: {e}")

if __name__ == "__main__":
    print("Starting Supabase Sync...")
    sync_activities()
    sync_prospects()
    sync_insights()
