#!/usr/bin/env python3
import asyncio
import json
import os
import re
import sys
import time
import urllib.request
import urllib.parse
from playwright.async_api import async_playwright

SUPABASE_URL = "https://jwswpryozfxzaocimadp.supabase.co"
SUPABASE_KEY = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Imp3c3dwcnlvemZ4emFvY2ltYWRwIiwicm9sZSI6ImFub24iLCJpYXQiOjE3ODYyNzQzMjQsImV4cCI6MjEwMTg1MDMyNH0.U575XIPsA12Y3JpZJ_gr9T7xH4WZafihkThQJvh8VNo"
PROCESSED_RESEARCH_FILE = "/home/adnan/x_bot/state/processed_research_leads.json"

os.makedirs("/home/adnan/x_bot/state", exist_ok=True)

TARGET_QUERIES = [
    'tella.tv',
    'screen.studio',
    'loom.com alternative',
    'need a product demo video',
    'how to make a product demo video',
    'looking for screen studio alternative',
    'shipped a new feature video',
    'showcase demo video SaaS'
]

def load_processed_leads():
    if os.path.exists(PROCESSED_RESEARCH_FILE):
        try:
            with open(PROCESSED_RESEARCH_FILE) as f:
                return set(json.load(f))
        except Exception:
            pass
    return set()

def save_processed_lead(handle):
    processed = load_processed_leads()
    processed.add(handle.lower())
    with open(PROCESSED_RESEARCH_FILE, "w") as f:
        json.dump(list(processed), f)

def calculate_lead_score(text, handle, url):
    score = 5
    if url:
        score += 2
    text_lower = text.lower()
    if any(k in text_lower for k in ['tella', 'screen studio', 'loom', 'guidde']):
        score += 2
    if any(k in text_lower for k in ['need', 'looking for', 'alternative', 'how to']):
        score += 1
    return min(score, 10)

def generate_personalized_hook(handle, url, text):
    if url:
        return f"Hey {handle}, saw your post regarding {url}! Created a 60s AI video demo walkthrough for {url} using PITCH — check it out on pitch.co/demo!"
    return f"Hey {handle}, saw you discussing SaaS video demos! PITCH automatically generates 1080p narrated product walkthroughs from any URL in 60s."

def upsert_supabase_prospect(handle, name, segment, url, notes, score):
    url_endpoint = f"{SUPABASE_URL}/rest/v1/prospects"
    payload = [{
        "handle": handle,
        "name": name or handle,
        "segment": segment,
        "stage": "new",
        "account": "@adnanspitch",
        "product_url": url or "",
        "notes": notes,
        "score": score,
        "touches": 0
    }]
    
    req = urllib.request.Request(url_endpoint, data=json.dumps(payload).encode("utf-8"), headers={
        "apikey": SUPABASE_KEY,
        "Authorization": f"Bearer {SUPABASE_KEY}",
        "Content-Type": "application/json",
        "Prefer": "resolution=merge-duplicates"
    }, method="POST")
    
    try:
        with urllib.request.urlopen(req) as resp:
            print(f"[Research Lead Saved to Supabase CRM]: {handle} (Score: {score}/10)")
            return True
    except Exception as e:
        print(f"[Supabase Prospect Error]: {e}")
        return False

async def run_247_researcher():
    state_file = "/home/adnan/x_bot/.browser-profile-adnanspitch/storageState.json"
    print("=== STARTING 24/7 BACKGROUND LEAD RESEARCHER DAEMON ===")
    
    async with async_playwright() as p:
        browser = await p.chromium.launch(headless=True)
        context = await browser.new_context(storage_state=state_file)
        page = await context.new_page()
        
        query_index = 0
        
        while True:
            query = TARGET_QUERIES[query_index % len(TARGET_QUERIES)]
            query_index += 1
            
            print(f"\n[24/7 Lead Researching Query]: '{query}'...")
            
            try:
                search_url = f"https://x.com/search?q={urllib.parse.quote(query)}&f=live"
                await page.goto(search_url, wait_until="domcontentloaded")
                await page.wait_for_timeout(3000)
                
                tweets = await page.locator('article[data-testid="tweet"]').all()
                processed = load_processed_leads()
                
                for tweet_el in tweets[:6]:
                    text = await tweet_el.inner_text()
                    lines = text.split("\n")
                    
                    user_handle = None
                    for l in lines:
                        if l.startswith("@") and "adnanspitch" not in l.lower() and "trypitchdotco" not in l.lower():
                            user_handle = l.strip()
                            break
                            
                    if not user_handle or user_handle.lower() in processed:
                        continue
                        
                    # Extract URL
                    url_match = re.search(r'https?://[^\s]+', text) or re.search(r'([a-zA-Z0-9-]+\.[a-zA-Z]{2,}(?:\/[^\s]*)?)', text)
                    target_url = url_match.group(0) if url_match else ""
                    if target_url and not target_url.startswith("http"):
                        target_url = f"https://{target_url}"
                        
                    score = calculate_lead_score(text, user_handle, target_url)
                    hook = generate_personalized_hook(user_handle, target_url, text)
                    
                    print(f"  -> Discovered Lead: {user_handle} | URL: {target_url} | Score: {score}/10")
                    save_processed_lead(user_handle)
                    
                    # Upsert prospect into Supabase CRM
                    notes = f"Discovered via search query '{query}'. Pre-cooked pitch hook: {hook}"
                    upsert_supabase_prospect(user_handle, user_handle, "founder", target_url, notes, score)
                    
            except Exception as e:
                print(f"[Research Loop Exception]: {e}")
                
            # Sleep 45 seconds between research queries (gentle, read-only rate)
            await asyncio.sleep(45)

if __name__ == "__main__":
    asyncio.run(run_247_researcher())
