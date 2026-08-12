#!/usr/bin/env python3
import sqlite3
import urllib.request
import json
import time
import os

DB_PATH = "/home/adnan/x_bot/data/pitch_bot.db"
SYNC_URL = "https://dashboard-blue-five-75.vercel.app/api/sync"

def do_sync():
    if not os.path.exists(DB_PATH):
        return

    try:
        conn = sqlite3.connect(DB_PATH)
        conn.row_factory = sqlite3.Row
        c = conn.cursor()

        # 1. Read mention_jobs
        c.execute("SELECT * FROM mention_jobs")
        jobs = [dict(r) for r in c.fetchall()]

        # 2. Read prospects
        c.execute("SELECT * FROM prospects")
        prospects = [dict(r) for r in c.fetchall()]

        # 3. Read activities
        c.execute("SELECT * FROM activities ORDER BY id DESC LIMIT 50")
        activities = [dict(r) for r in c.fetchall()]

        conn.close()

        payload = json.dumps({
            "mention_jobs": jobs,
            "prospects": prospects,
            "activities": activities
        }).encode("utf-8")

        req = urllib.request.Request(SYNC_URL, data=payload, headers={"Content-Type": "application/json"}, method="POST")
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            # Quiet success log
            pass
    except Exception as e:
        print(f"[SQLite -> Dashboard Sync Warning]: {e}")

if __name__ == "__main__":
    while True:
        do_sync()
        time.sleep(10)
