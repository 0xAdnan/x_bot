import asyncio
import json
from playwright.async_api import async_playwright

STORAGE_STATE = "/home/adnan/x_bot/.browser-profile-trypitchdotco/storageState_trypitchdotco.json"

async def scan():
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

        queries = [
            "polymarket ai",
            "demo video saas",
            "building ai devtool",
            "product hunt demo"
        ]

        found_tweets = []

        for q in queries:
            url = f"https://x.com/search?q={q}&f=live"
            print(f"\nSearching: {url}")
            try:
                await page.goto(url, wait_until="domcontentloaded", timeout=20000)
                await page.wait_for_timeout(4000)

                articles = await page.query_selector_all('article[data-testid="tweet"]')
                print(f"Found {len(articles)} tweets for '{q}'")

                for art in articles[:5]:
                    text = await art.inner_text()
                    links = await art.query_selector_all('a[href*="/status/"]')
                    tweet_url = ""
                    author_handle = ""
                    for link in links:
                        href = await link.get_attribute("href")
                        if href and "/status/" in href:
                            tweet_url = f"https://x.com{href}" if href.startswith("/") else href
                            parts = href.strip("/").split("/")
                            if len(parts) >= 1:
                                author_handle = f"@{parts[0]}"
                            break

                    lines = [line.strip() for line in text.split("\n") if line.strip()]
                    snippet = " ".join(lines[:4])
                    if tweet_url:
                        found_tweets.append({
                            "query": q,
                            "author": author_handle,
                            "url": tweet_url,
                            "snippet": snippet
                        })
            except Exception as e:
                print(f"Search error for '{q}':", e)

        print("\n--- CANDIDATE TWEETS ---")
        for t in found_tweets:
            print(f"[{t['query']}] {t['author']}: {t['snippet'][:100]}... -> {t['url']}")

        await browser.close()

if __name__ == "__main__":
    asyncio.run(scan())
