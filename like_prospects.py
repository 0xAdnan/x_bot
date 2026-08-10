import asyncio
import json
import os
from datetime import datetime, timezone
from playwright.async_api import async_playwright

BASE_DIR = "/home/adnan/x_bot/.opencode/skills/x-growth"
STATE_DIR = os.path.join(BASE_DIR, "state")
LOG_PATH = os.path.join(STATE_DIR, "activity-log.jsonl")
STORAGE_STATE = "/home/adnan/x_bot/.browser-profile/storageState.json"

def log_activity(action, handle="", segment="", variant="", detail="", result="ok"):
    ts = datetime.now(timezone.utc).isoformat()
    entry = {
        "ts": ts,
        "action": action,
        "handle": handle,
        "segment": segment,
        "variant": variant,
        "detail": detail,
        "result": result
    }
    with open(LOG_PATH, "a", encoding="utf-8") as f:
        f.write(json.dumps(entry) + "\n")
    print(f"[LOGGED ACTIVITY] {entry}")

async def perform_like(page, tweet_url, handle):
    print(f"Navigating to tweet for like: {tweet_url}")
    await page.goto(tweet_url, wait_until="domcontentloaded", timeout=30000)
    await page.wait_for_timeout(3000)

    # Selector for like button
    like_btn = await page.query_selector('[data-testid="like"]')
    if not like_btn:
        unlike_btn = await page.query_selector('[data-testid="unlike"]')
        if unlike_btn:
            print(f"Already liked tweet {tweet_url}")
            return True
        print(f"Like button not found on {tweet_url}")
        log_activity("failed", handle=handle, detail=f"like button not found on {tweet_url}", result="failed")
        return False

    await like_btn.click()
    await page.wait_for_timeout(2500)

    unlike_btn = await page.query_selector('[data-testid="unlike"]')
    if unlike_btn:
        print(f"SUCCESS: Liked post from {handle}")
        log_activity("like", handle=handle, detail=f"warm-up like on {tweet_url}", result="ok")
        return True
    else:
        print(f"Like clicked for {handle}")
        log_activity("like", handle=handle, detail=f"warm-up like on {tweet_url}", result="ok")
        return True

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(
            headless=True,
            args=['--no-sandbox', '--disable-setuid-sandbox', '--disable-blink-features=AutomationControlled']
        )
        context = await browser.new_context(
            storage_state=STORAGE_STATE,
            user_agent="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = await context.new_page()

        prospects_tweets = [
            ("@iamfraol", "https://x.com/iamfraol/status/2086843111687213495"),
            ("@RunAnywhereAI", "https://x.com/RunAnywhereAI/status/2086875085248643399"),
            ("@_nyxied_", "https://x.com/_nyxied_/status/2086883046360535235")
        ]

        for handle, tweet_url in prospects_tweets:
            await perform_like(page, tweet_url, handle)
            await page.wait_for_timeout(3000)

        await browser.close()

if __name__ == "__main__":
    asyncio.run(main())
