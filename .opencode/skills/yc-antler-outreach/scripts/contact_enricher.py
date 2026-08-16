#!/usr/bin/env python3
"""
Website Contact & Metadata Enricher
Extracts emails, social links (Twitter/X, LinkedIn, GitHub), founder names,
and product positioning from startup websites.
"""

import re
import json
import urllib.request
import urllib.parse
from html import unescape
from typing import Dict, List, Any, Optional

IGNORED_EMAIL_DOMAINS = {
    "example.com", "domain.com", "email.com", "wixpress.com", "sentry.io",
    "wix.com", "github.com", "google.com", "apple.com", "cloudflare.com",
    "schema.org", "2x.png", "2x.jpg", "png", "jpg", "jpeg", "webp"
}

IGNORED_TWITTER_HANDLES = {
    "twitter", "x", "intent", "share", "home", "search", "explore",
    "notifications", "messages", "privacy", "tos", "help", "about",
    "hashtag", "i", "settings"
}

def clean_url(url: str) -> str:
    if not url:
        return ""
    if not url.startswith("http://") and not url.startswith("https://"):
        return f"https://{url}"
    return url

def fetch_html(url: str, timeout: int = 7) -> str:
    headers = {
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"
    }
    try:
        req = urllib.request.Request(clean_url(url), headers=headers)
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            content_type = resp.headers.get("Content-Type", "")
            if "text/html" in content_type or "text/plain" in content_type:
                return resp.read().decode("utf-8", errors="ignore")
    except Exception:
        pass
    return ""

def extract_emails(text: str, domain: str = "") -> List[str]:
    """Extract valid email addresses from text/HTML."""
    # Clean unicode escapes or url encoding
    clean_text = text.replace(r"\u003c", "<").replace(r"\u003e", ">").replace("&lt;", "<").replace("&gt;", ">")
    clean_text = re.sub(r'u003[ce]', '', clean_text, flags=re.I)
    raw_emails = re.findall(r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}', clean_text)
    valid = []
    seen = set()
    
    clean_domain = domain.replace("https://", "").replace("http://", "").replace("www.", "").split("/")[0].lower()
    
    for email in raw_emails:
        email_clean = email.lower().strip()
        parts = email_clean.split("@")
        if len(parts) != 2:
            continue
        user, dom = parts
        
        # Filter junk/assets
        if dom in IGNORED_EMAIL_DOMAINS or any(ext in dom for ext in [".png", ".jpg", ".svg", ".avif", ".webp"]):
            continue
        if len(user) < 2 or len(dom) < 4:
            continue
            
        if email_clean not in seen:
            seen.add(email_clean)
            # Prioritize company domain or founder/hello/team emails
            if clean_domain and clean_domain in dom:
                valid.insert(0, email_clean)
            else:
                valid.append(email_clean)
                
    return valid

def extract_twitter_handles(text: str) -> List[str]:
    """Extract Twitter / X handles from links or text."""
    matches = re.findall(r'(?:twitter\.com|x\.com)/([a-zA-Z0-9_]{1,25})', text, re.IGNORECASE)
    valid = []
    seen = set()
    for m in matches:
        handle = m.lower()
        if handle not in IGNORED_TWITTER_HANDLES and handle not in seen:
            seen.add(handle)
            valid.append(f"@{m}")
    return valid

def extract_meta_info(html: str) -> Dict[str, str]:
    """Extract page title, description, and keywords."""
    info = {"title": "", "description": "", "keywords": ""}
    
    title_m = re.search(r'<title[^>]*>(.*?)</title>', html, re.I | re.DOTALL)
    if title_m:
        info["title"] = unescape(title_m.group(1).strip())
        
    desc_m = re.search(r'<meta[^>]+(?:name|property)=[\"\'](?:description|og:description)[\"\'][^>]+content=[\"\']([^\"\']+)[\"\']', html, re.I)
    if not desc_m:
        desc_m = re.search(r'<meta[^>]+content=[\"\']([^\"\']+)[\"\'][^>]+(?:name|property)=[\"\'](?:description|og:description)[\"\']', html, re.I)
    if desc_m:
        info["description"] = unescape(desc_m.group(1).strip())
        
    return info

def enrich_startup(website_url: str, base_info: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
    """Deeply inspect startup website and subpages to gather emails, socials, and product hooks."""
    enriched = {
        "website": website_url,
        "emails": [],
        "primary_email": "",
        "twitter_handles": [],
        "primary_handle": "",
        "title": "",
        "description": "",
        "demo_hook": ""
    }
    
    if base_info:
        enriched.update(base_info)
        
    if not website_url:
        return enriched
        
    main_html = fetch_html(website_url)
    if not main_html:
        return enriched
        
    meta = extract_meta_info(main_html)
    enriched["title"] = meta["title"]
    if not enriched.get("description"):
        enriched["description"] = meta["description"]
        
    emails = extract_emails(main_html, website_url)
    twitters = extract_twitter_handles(main_html)
    
    # Try fetching /about or /contact or /team
    parsed = urllib.parse.urlparse(clean_url(website_url))
    base = f"{parsed.scheme}://{parsed.netloc}"
    
    for path in ["/about", "/contact", "/team", "/privacy"]:
        sub_url = f"{base}{path}"
        sub_html = fetch_html(sub_url, timeout=4)
        if sub_html:
            for e in extract_emails(sub_html, website_url):
                if e not in emails:
                    emails.append(e)
            for t in extract_twitter_handles(sub_html):
                if t not in twitters:
                    twitters.append(t)
                    
    enriched["emails"] = emails
    enriched["primary_email"] = emails[0] if emails else ""
    
    # If base_info had twitter handles, merge
    if base_info and base_info.get("twitter_handles"):
        for h in base_info["twitter_handles"]:
            if h not in twitters:
                twitters.append(h)
                
    enriched["twitter_handles"] = twitters
    enriched["primary_handle"] = twitters[0] if twitters else ""
    
    return enriched

if __name__ == "__main__":
    test_url = "https://wideframe.com"
    print(f"[*] Enriching {test_url}...")
    res = enrich_startup(test_url, {"name": "Wideframe", "batch": "Winter 2026"})
    print("Enriched result:", json.dumps(res, indent=2))
