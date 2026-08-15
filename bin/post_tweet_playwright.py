#!/usr/bin/env python3
"""
post_tweet_playwright.py — Enhanced multi-account tweet, quote-tweet, reply, and thread publisher
for @trypitchdotco and @adnanspitch using real Playwright browser sessions.
"""

import sys
import os
import json
import asyncio
import urllib.request
import re
from playwright.async_api import async_playwright

REPO_DIR = "/home/adnan/x_bot"

def get_storage_state(account):
    clean_acc = account.replace("@", "").lower()
    if clean_acc == "trypitchdotco" or clean_acc == "trypitch":
        candidates = [
            os.path.join(REPO_DIR, ".browser-profile", "storageState_trypitchdotco.json"),
            os.path.join(REPO_DIR, ".browser-profile-trypitchdotco", "storageState_trypitchdotco.json"),
            os.path.join(REPO_DIR, ".browser-profile", "storageState_trypitch_co.json"),
            os.path.join(REPO_DIR, ".browser-profile", "storageState.json")
        ]
    else: # adnanspitch / operator
        candidates = [
            os.path.join(REPO_DIR, ".browser-profile-adnanspitch", "storageState.json"),
            os.path.join(REPO_DIR, ".browser-profile", "storageState.json")
        ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return candidates[0]

def download_media_if_needed(image_url):
    if not image_url or not image_url.startswith("http"):
        return None
    try:
        local_img = "/tmp/publish_attachment.png"
        headers = {"User-Agent": "Mozilla/5.0"}
        if "4cdn.org" in image_url:
            headers["Referer"] = "https://boards.4channel.org/"
        req = urllib.request.Request(image_url, headers=headers)
        with urllib.request.urlopen(req, timeout=10) as resp, open(local_img, "wb") as f:
            f.write(resp.read())
        return local_img
    except Exception as e:
        print(f"[Warn] Could not download image attachment: {e}")
        return None

async def publish_tweet(account, text, image_url=None, reply_to_url=None, quote_url=None):
    storage_state = get_storage_state(account)
    clean_handle = account.replace("@", "").lower()
    print(f"[Publish] Account: {account} | State: {storage_state}")
    
    local_img = download_media_if_needed(image_url) if image_url else None
    
    # If quote_url is specified and not already in text, append it so X renders Quote Tweet card
    final_text = text.strip()
    if quote_url and quote_url not in final_text:
        final_text = f"{final_text}\n\n{quote_url}"

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(
            storage_state=storage_state if os.path.exists(storage_state) else None,
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = await context.new_page()

        # If replying directly to a tweet URL
        if reply_to_url and "status" in reply_to_url:
            print(f"[Publish] Navigating to reply target: {reply_to_url}...")
            await page.goto(reply_to_url, wait_until="domcontentloaded")
            await page.wait_for_timeout(4000)

            # Dwell & Scroll
            await page.evaluate("window.scrollBy(0, 300)")
            await page.wait_for_timeout(2000)
            await page.evaluate("window.scrollBy(0, -300)")
            await page.wait_for_timeout(1000)

            reply_box = page.locator('div[data-testid="tweetTextarea_0"]').first
            if await reply_box.count() == 0:
                reply_box = page.locator('div[role="textbox"]').first

            if await reply_box.count() > 0:
                await reply_box.click()
                await page.wait_for_timeout(800)
                await reply_box.fill(final_text)
                await page.wait_for_timeout(2000)

                reply_btn = page.locator('button[data-testid="tweetButtonInline"]').first
                if await reply_btn.count() == 0:
                    reply_btn = page.locator('button[data-testid="tweetButton"]').first

                if await reply_btn.count() > 0:
                    await reply_btn.click()
                    await page.wait_for_timeout(3000)
                    
                    # Extract toast link if available
                    reply_url_out = f"https://x.com/{clean_handle}"
                    try:
                        toast = page.locator('div[data-testid="toast"] a[href*="/status/"]').first
                        if await toast.count() > 0:
                            href = await toast.get_attribute("href")
                            if href:
                                reply_url_out = f"https://x.com{href}" if href.startswith('/') else href
                    except Exception:
                        pass

                    await browser.close()
                    return {
                        "success": True,
                        "account": account,
                        "text": final_text,
                        "type": "reply",
                        "message": f"Successfully replied as {account}!",
                        "url": reply_url_out
                    }
        
        # Standard Compose / Quote Tweet
        print("[Publish] Navigating to https://x.com/compose/post...")
        await page.goto("https://x.com/compose/post", wait_until="domcontentloaded")
        await page.wait_for_timeout(3000)

        if "login" in page.url:
            await browser.close()
            return {"success": False, "error": f"Session expired for {account}. Please re-authenticate."}

        # Attach image if present
        if local_img and os.path.exists(local_img):
            file_input = page.locator('input[data-testid="fileInput"]').first
            if await file_input.count() > 0:
                print(f"[Publish] Uploading media attachment: {local_img}...")
                await file_input.set_input_files(local_img)
                await page.wait_for_timeout(4000)

        # Focus compose box
        editor = page.locator('div[data-testid="tweetTextarea_0"]').first
        if await editor.count() == 0:
            editor = page.locator('div[role="textbox"]').first

        if await editor.count() == 0:
            await browser.close()
            return {"success": False, "error": "Could not locate compose textbox"}

        await editor.click()
        await page.wait_for_timeout(500)
        await editor.fill(final_text)
        await page.wait_for_timeout(2000)

        # Click Post button
        post_btn = page.locator('button[data-testid="tweetButton"]').first
        if await post_btn.count() == 0:
            post_btn = page.locator('button[data-testid="tweetButtonInline"]').first

        if await post_btn.count() > 0:
            print("[Publish] Clicking Post button...")
            await post_btn.click()
            await page.wait_for_timeout(3000)
            
            post_url_out = f"https://x.com/{clean_handle}"
            try:
                toast = page.locator('div[data-testid="toast"] a[href*="/status/"]').first
                if await toast.count() > 0:
                    href = await toast.get_attribute("href")
                    if href:
                        post_url_out = f"https://x.com{href}" if href.startswith('/') else href
            except Exception:
                pass

            await browser.close()
            return {
                "success": True,
                "account": account,
                "text": final_text,
                "type": "quote" if quote_url else "post",
                "message": f"Successfully published {'quote tweet' if quote_url else 'post'} as {account}!",
                "url": post_url_out
            }
        else:
            await browser.close()
            return {"success": False, "error": "Post button not clickable"}

async def publish_thread(account, tweets):
    """
    Publishes a multi-tweet thread sequentially.
    tweets = ["Tweet 1 text...", "Tweet 2 text...", "Tweet 3 text..."]
    """
    if not tweets:
        return {"success": False, "error": "Empty thread array"}

    storage_state = get_storage_state(account)
    clean_handle = account.replace("@", "").lower()
    print(f"[Thread Publish] Account: {account} | Total Tweets: {len(tweets)}")

    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(
            storage_state=storage_state if os.path.exists(storage_state) else None,
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = await context.new_page()

        # Step 1: Post Tweet 1
        await page.goto("https://x.com/compose/post", wait_until="domcontentloaded")
        await page.wait_for_timeout(3000)

        editor = page.locator('div[data-testid="tweetTextarea_0"]').first
        if await editor.count() == 0:
            editor = page.locator('div[role="textbox"]').first

        if await editor.count() == 0:
            await browser.close()
            return {"success": False, "error": "Could not locate compose textbox"}

        # Use the X "+ Add Tweet" button inside the composer if multiple tweets exist
        for i, tweet_text in enumerate(tweets):
            if i > 0:
                add_btn = page.locator('button[data-testid="addButton"]').first
                if await add_btn.count() > 0:
                    await add_btn.click()
                    await page.wait_for_timeout(1000)

            # Get the current textarea
            textareas = page.locator('div[data-testid^="tweetTextarea_"]')
            cnt = await textareas.count()
            target_editor = textareas.nth(cnt - 1) if cnt > 0 else page.locator('div[role="textbox"]').last

            await target_editor.click()
            await page.wait_for_timeout(300)
            await target_editor.fill(tweet_text.strip())
            await page.wait_for_timeout(800)

        # Post all at once via "Post All" button
        post_all_btn = page.locator('button[data-testid="tweetButton"]').first
        if await post_all_btn.count() == 0:
            post_all_btn = page.locator('button[data-testid="tweetButtonInline"]').first

        if await post_all_btn.count() > 0:
            print(f"[Thread Publish] Clicking Post All button for {len(tweets)} tweets...")
            await post_all_btn.click()
            await page.wait_for_timeout(5000)
            await browser.close()
            return {
                "success": True,
                "account": account,
                "total_tweets": len(tweets),
                "type": "thread",
                "message": f"Successfully published {len(tweets)}-tweet thread as {account}!",
                "url": f"https://x.com/{clean_handle}"
            }
        else:
            await browser.close()
            return {"success": False, "error": "Post All button not found"}

def main():
    if len(sys.argv) < 3:
        print("Usage:")
        print("  Single:  post_tweet_playwright.py <@account> <text> [image_url] [reply_to_url] [quote_url]")
        print("  Thread:  post_tweet_playwright.py <@account> --thread '<json_array_of_tweets>'")
        sys.exit(1)

    account = sys.argv[1]

    if sys.argv[2] == "--thread":
        tweets_json = sys.argv[3]
        try:
            tweets_list = json.loads(tweets_json)
        except Exception:
            tweets_list = [t.strip() for t in tweets_json.split("\n\n") if t.strip()]
        res = asyncio.run(publish_thread(account, tweets_list))
        print(json.dumps(res))
        return

    text = sys.argv[2]
    img = sys.argv[3] if len(sys.argv) > 3 and sys.argv[3] != "null" else None
    reply_url = sys.argv[4] if len(sys.argv) > 4 and sys.argv[4] != "null" else None
    quote_url = sys.argv[5] if len(sys.argv) > 5 and sys.argv[5] != "null" else None

    res = asyncio.run(publish_tweet(account, text, image_url=img, reply_to_url=reply_url, quote_url=quote_url))
    print(json.dumps(res))

if __name__ == "__main__":
    main()
