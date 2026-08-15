#!/usr/bin/env python3
"""
post_tweet_playwright.py — Reliable tweet publisher for @trypitchdotco and @adnanspitch.
Supports text and image attachments.
"""

import sys
import os
import json
import asyncio
import urllib.request
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

async def publish(account, text, image_url=None, media_path=None):
    storage_state = get_storage_state(account)
    print(f"[Publish] Account: {account} | State: {storage_state}")
    
    # Download image if remote URL
    local_img = None
    if media_path and os.path.exists(media_path):
        local_img = media_path
    elif image_url and image_url.startswith("http"):
        try:
            local_img = "/tmp/publish_attachment.png"
            req = urllib.request.Request(image_url, headers={"User-Agent": "Mozilla/5.0"})
            with urllib.request.urlopen(req, timeout=10) as resp, open(local_img, "wb") as f:
                f.write(resp.read())
        except Exception as e:
            print(f"[Warn] Could not download image attachment: {e}")
            local_img = None

    async with async_playwright() as p:
        # Connect or launch
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(
            storage_state=storage_state if os.path.exists(storage_state) else None,
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = await context.new_page()

        print("[Publish] Navigating to https://x.com/compose/post...")
        await page.goto("https://x.com/compose/post", wait_until="domcontentloaded")
        await page.wait_for_timeout(3000)

        # Check if redirected to login
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
        await editor.fill(text)
        await page.wait_for_timeout(2000)

        # Click Post button
        post_btn = page.locator('button[data-testid="tweetButton"]').first
        if await post_btn.count() == 0:
            post_btn = page.locator('button[data-testid="tweetButtonInline"]').first

        if await post_btn.count() > 0:
            print("[Publish] Clicking Post button...")
            await post_btn.click()
            await page.wait_for_timeout(4000)
            
            clean_handle = account.replace("@", "").lower()
            await browser.close()
            return {
                "success": True,
                "account": account,
                "text": text,
                "message": f"Successfully published tweet as {account}!",
                "url": f"https://x.com/{clean_handle}"
            }
        else:
            await browser.close()
            return {"success": False, "error": "Post button not clickable"}

def main():
    if len(sys.argv) < 3:
        print("Usage: post_tweet_playwright.py <@account> <text> [image_url_or_path]")
        sys.exit(1)

    account = sys.argv[1]
    text = sys.argv[2]
    img = sys.argv[3] if len(sys.argv) > 3 else None

    res = asyncio.run(publish(account, text, image_url=img))
    print(json.dumps(res))

if __name__ == "__main__":
    main()
