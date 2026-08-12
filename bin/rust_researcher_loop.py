#!/usr/bin/env python3
import time
import subprocess
import os

print("=== STARTING 24/7 AI RESEARCHER & LEAD PRIORITIZATION DAEMON ===")

cmd_env = os.environ.copy()
cmd_env["PATH"] = os.path.expanduser("~/.cargo/bin:") + cmd_env.get("PATH", "")

def run_research_pass():
    print(f"[{time.strftime('%Y-%m-%d %H:%M:%S')}] Executing 24/7 AI Research Pass on X...")
    try:
        res = subprocess.run(
            ["/home/adnan/x_bot/target/release/pitch-cli", "discover", "--max", "10"],
            capture_output=True,
            text=True,
            cwd="/home/adnan/x_bot",
            env=cmd_env
        )
        if res.stdout:
            print("[Researcher Output]:", res.stdout.strip())
    except Exception as e:
        print(f"[Researcher Error]: {e}")

if __name__ == "__main__":
    while True:
        run_research_pass()
        time.sleep(1800) # Run every 30 minutes 24/7
