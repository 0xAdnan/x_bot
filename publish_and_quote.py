import asyncio
import json
import os
import sys
from datetime import datetime, timezone
from playwright.async_api import async_playwright

STORAGE_STATE = "/home/adnan/x_bot/.browser-profile-trypitchdotco/storageState_trypitchdotco.json"
LOG_PATH = "/home/adnan/x_bot/.opencode/skills/x-growth/state/activity-log.jsonl"

def log_activity(action, handle="@trypitchdotco", segment="", variant="", detail="", result="ok"):
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

async def publish_original_post(page, post_text):
    print("\n--- STEP 3: PUBLISHING ORIGINAL PRODUCT POST ---")
    print(f"Post text:\n{post_text}\n")

    await page.goto("https://x.com/compose/post", wait_until="domcontentloaded", timeout=30000)
    await page.wait_for_timeout(4000)

    # Locate compose textarea
    textarea = await page.query_selector('div[data-testid="tweetTextarea_0"]')
    if not textarea:
        # Try going to home page
        await page.goto("https://x.com/home", wait_until="domcontentloaded", timeout=30000)
        await page.wait_for_timeout(4000)
        textarea = await page.query_selector('div[data-testid="tweetTextarea_0"]')

    if not textarea:
        print("ERROR: Compose textarea not found!")
        log_activity("post", detail=f"Product post failed: textarea not found", result="failed")
        return False

    print("Clicking and filling compose textarea...")
    await textarea.click()
    await page.wait_for_timeout(1000)
    await textarea.fill(post_text)
    await page.wait_for_timeout(2000)

    # Find Post button
    post_button = await page.query_selector('button[data-testid="tweetButton"]')
    if not post_button:
        post_button = await page.query_selector('button[data-testid="tweetButtonInline"]')

    if not post_button:
        print("ERROR: Post button not found!")
        log_activity("post", detail=f"Product post failed: post button not found", result="failed")
        return False

    print("Clicking Post button...")
    await post_button.click()
    await page.wait_for_timeout(6000)

    print("SUCCESS: Published original post!")
    log_activity("post", detail=f"Product post published: {post_text}", result="ok")
    return True

async def perform_quote_tweet(page, target_url, quote_text, target_handle):
    print("\n--- STEP 4: PUBLISHING QUOTE TWEET ---")
    print(f"Target: {target_url}")
    print(f"Quote text:\n{quote_text}\n")

    await page.goto(target_url, wait_until="domcontentloaded", timeout=30000)
    await page.wait_for_timeout(4000)

    # Locate retweet / repost button
    retweet_btn = await page.query_selector('button[data-testid="retweet"]')
    if not retweet_btn:
        retweet_btn = await page.query_selector('div[data-testid="retweet"]')

    if not retweet_btn:
        print("ERROR: Retweet button not found!")
        log_activity("quote", detail=f"trend quote failed on {target_url}: retweet button not found", result="failed")
        return False

    print("Clicking Retweet button...")
    await retweet_btn.click()
    await page.wait_for_timeout(2000)

    # Look for Quote option in dropdown menu
    # Common testids/selectors for quote tweet in menu:
    quote_menu_item = await page.query_selector('a[href="/compose/post"]')
    if not quote_menu_item:
        # Search menu items by text "Quote"
        menu_items = await page.query_selector_all('div[role="menuitem"]')
        for item in menu_items:
            text = await item.inner_text()
            if "Quote" in text:
                quote_menu_item = item
                break

    if not quote_menu_item:
        # Try finding quote menu item by role or text
        menu_items = await page.query_selector_all('[role="menuitem"]')
        for item in menu_items:
            text = await item.inner_text()
            if "Quote" in text:
                quote_menu_item = item
                break

    if not quote_menu_item:
        print("ERROR: Quote option not found in menu!")
        log_activity("quote", detail=f"trend quote failed on {target_url}: quote menu option not found", result="failed")
        return False

    print("Clicking Quote option...")
    await quote_menu_item.click()
    await page.wait_for_timeout(3000)

    # Locate quote compose textarea
    textarea = await page.query_selector('div[data-testid="tweetTextarea_0"]')
    if not textarea:
        print("ERROR: Quote textarea not found!")
        log_activity("quote", detail=f"trend quote failed on {target_url}: quote textarea not found", result="failed")
        return False

    print("Filling quote text...")
    await textarea.click()
    await page.wait_for_timeout(1000)
    await textarea.fill(quote_text)
    await page.wait_for_timeout(2000)

    # Find post button for quote
    post_button = await page.query_selector('button[data-testid="tweetButton"]')
    if not post_button:
        post_button = await page.query_selector('button[data-testid="tweetButtonInline"]')

    if not post_button:
        print("ERROR: Quote post button not found!")
        log_activity("quote", detail=f"trend quote failed on {target_url}: quote post button not found", result="failed")
        return False

    print("Clicking Post button for quote...")
    await post_button.click()
    await page.wait_for_timeout(6000)

    print("SUCCESS: Published quote tweet!")
    detail_msg = f"Quoted {target_handle} ({target_url}): {quote_text}"
    log_activity("quote", detail=detail_msg, result="ok")
    return True

async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch(
            headless=True,
            args=["--no-sandbox", "--disable-setuid-sandbox", "--disable-blink-features=AutomationControlled"]
        )
        context = await browser.new_context(
            storage_state=STORAGE_STATE,
            user_agent="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = await context.new_page()

        # Step 3: Product post
        post_text = "the worst product demo opens on a login screen or an empty dashboard.\n\nthe best ones open on the outcome your user actually came for.\n\nwe built @trypitchdotco so you describe the walkthrough in text and get a narrated studio-quality video back in minutes. https://trypitch.co"
        post_ok = await publish_original_post(page, post_text)

        await page.wait_for_timeout(5000)

        # Step 4: Trend Quote Tweet on @hustle_fred's devtool SaaS thread
        target_url = "https://x.com/hustle_fred/status/2086481164949168354"
        target_handle = "@hustle_fred"
        quote_text = "item 79 hit way too close to home. UI changes triggering full demo re-records is why we built @trypitchdotco. describe the walkthrough, get a narrated video in minutes."
        quote_ok = await perform_quote_tweet(page, target_url, quote_text, target_handle)

        await browser.close()
        print("\nPublishing and Quote Tweet finished!")

if __name__ == "__main__":
    asyncio.run(main())
