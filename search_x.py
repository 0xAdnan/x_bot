import asyncio
import json
import urllib.parse
from playwright.async_api import async_playwright

STORAGE_STATE = "/home/adnan/x_bot/.browser-profile/storageState.json"

queries = [
    '("launching today" OR "just launched") (SaaS OR AI OR devtool) -filter:replies',
    '("YC S26" OR "YC W26" OR "YC S25") -filter:replies',
    '("building in public" OR "indie hacker") (demo OR launch OR video) -filter:replies'
]

async def search_tweets():
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

        results = []
        for q in queries:
            url = f"https://x.com/search?q={urllib.parse.quote(q)}&f=live"
            print(f"\nSearching URL: {url}")
            await page.goto(url, wait_until="domcontentloaded", timeout=30000)
            await page.wait_for_timeout(4000)

            tweets = await page.query_selector_all('article[data-testid="tweet"]')
            print(f"Found {len(tweets)} tweets for query '{q}'")

            for tweet in tweets[:6]:
                try:
                    # Extract handle, tweet link, text
                    user_elem = await tweet.query_selector('div[data-testid="User-Name"]')
                    text_elem = await tweet.query_selector('div[data-testid="tweetText"]')
                    time_elem = await tweet.query_selector('time')
                    
                    user_text = await user_elem.inner_text() if user_elem else ""
                    tweet_text = await text_elem.inner_text() if text_elem else ""
                    
                    # Get link
                    link_elem = await tweet.query_selector('a[href*="/status/"]')
                    tweet_url = ""
                    if link_elem:
                        href = await link_elem.get_attribute("href")
                        if href:
                            tweet_url = f"https://x.com{href}" if href.startswith("/") else href

                    lines = [line.strip() for line in user_text.split('\n') if line.strip()]
                    name = lines[0] if len(lines) > 0 else ""
                    handle = ""
                    for l in lines:
                        if l.startswith("@"):
                            handle = l
                            break

                    if handle and tweet_url and tweet_text:
                        results.append({
                            "query": q,
                            "handle": handle,
                            "name": name,
                            "tweet_url": tweet_url,
                            "text": tweet_text.replace("\n", " ")
                        })
                except Exception as e:
                    print("Error parsing tweet:", e)

        print("\n--- RESULTS ---")
        print(json.dumps(results, indent=2))
        await browser.close()

if __name__ == "__main__":
    asyncio.run(search_tweets())
