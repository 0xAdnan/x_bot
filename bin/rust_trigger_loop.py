#!/usr/bin/env python3
import time
import subprocess
import os

print("=== STARTING CONTINUOUS 10s RUST TRIGGER LOOP ===")

cmd = [os.path.expanduser("~/.cargo/bin/cargo"), "run", "--release", "--", "trigger"]
env = os.environ.copy()
env["PATH"] = os.path.expanduser("~/.cargo/bin:") + env.get("PATH", "")

while True:
    try:
        res = subprocess.run(
            ["/home/adnan/x_bot/target/release/pitch-cli", "trigger"],
            capture_output=True,
            text=True,
            cwd="/home/adnan/x_bot",
            env=env
        )
        if res.stdout:
            lines = res.stdout.strip().split("\n")
            # Log summary
            summary = [l for l in lines if "Total" in l or "Mention Jobs" in l or "CRM" in l or "OK" in l]
            if summary:
                print(f"[Trigger Loop] {time.strftime('%H:%M:%S')} - {' | '.join(summary)}")
    except Exception as e:
        print(f"[Trigger Loop Error]: {e}")
    time.sleep(10)
