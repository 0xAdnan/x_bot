#!/usr/bin/env python3
"""
Continuous 24/7 YC & Antler Startup Scout Daemon
Periodically discovers new YC and Antler cohort startups, enriches founder contacts,
crafts anti-AI demo/launch video pitches, and syncs directly into SQLite CRM memory.
"""

import time
import subprocess
import os
import sys
from pathlib import Path

REPO_ROOT = Path("/home/adnan/x_bot")
SCRIPT_PATH = REPO_ROOT / ".opencode" / "skills" / "yc-antler-outreach" / "scripts" / "scout_enrich.py"
LOG_DIR = REPO_ROOT / "data" / "logs"
LOG_DIR.mkdir(parents=True, exist_ok=True)
LOG_FILE = LOG_DIR / "pass-yc-antler.log"

print("=== STARTING CONTINUOUS YC & ANTLER STARTUP SCOUT DAEMON ===")

INTERVAL_SECONDS = int(os.environ.get("YC_ANTLER_INTERVAL", 900)) # Default 15 mins

def run_scout_cycle():
    timestamp = time.strftime("%Y-%m-%d %H:%M:%S")
    msg = f"\n[{timestamp}] Executing YC & Antler Batch Scout Pass...\n"
    print(msg)
    with open(LOG_FILE, "a", encoding="utf-8") as lf:
        lf.write(msg)
        
    try:
        proc = subprocess.run(
            [sys.executable, str(SCRIPT_PATH), "--source", "all", "--limit", "8"],
            capture_output=True,
            text=True,
            cwd=str(REPO_ROOT)
        )
        if proc.stdout:
            print(proc.stdout)
            with open(LOG_FILE, "a", encoding="utf-8") as lf:
                lf.write(proc.stdout + "\n")
        if proc.stderr:
            print(f"[Scout Warning]: {proc.stderr}", file=sys.stderr)
            with open(LOG_FILE, "a", encoding="utf-8") as lf:
                lf.write(f"[Error]: {proc.stderr}\n")
    except Exception as e:
        err_msg = f"[{timestamp}] [Scout Exception]: {e}\n"
        print(err_msg, file=sys.stderr)
        with open(LOG_FILE, "a", encoding="utf-8") as lf:
            lf.write(err_msg)

if __name__ == "__main__":
    while True:
        run_scout_cycle()
        print(f"[{time.strftime('%H:%M:%S')}] Sleeping {INTERVAL_SECONDS}s until next cohort discovery cycle...")
        time.sleep(INTERVAL_SECONDS)
