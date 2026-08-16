#!/usr/bin/env python3
"""
YC Startup Scout & Contact Extractor
Fetches upcoming & recent YC startups (e.g. Winter 2026, Summer 2025, Winter 2025)
along with founders, Twitter/X handles, website, and product descriptions.
"""

import sys
import os
import re
import json
import urllib.request
import urllib.parse
from html import unescape
from typing import List, Dict, Any, Optional

DEFAULT_APP_ID = "45BWZJ1SGC"
DEFAULT_API_KEY = "NzllNTY5MzJiZGM2OTY2ZTQwMDEzOTNhYWZiZGRjODlhYzVkNjBmOGRjNzJiMWM4ZTU0ZDlhYTZjOTJiMjlhMWFuYWx5dGljc1RhZ3M9eWNkYyZyZXN0cmljdEluZGljZXM9WUNDb21wYW55X3Byb2R1Y3Rpb24lMkNZQ0NvbXBhbnlfQnlfTGF1bmNoX0RhdGVfcHJvZHVjdGlvbiZ0YWdGaWx0ZXJzPSU1QiUyMnljZGNfcHVibGljJTIyJTVE"

def get_algolia_creds() -> tuple[str, str]:
    """Fetch live Algolia credentials from ycombinator.com/companies if available, else use default."""
    try:
        req = urllib.request.Request("https://www.ycombinator.com/companies", headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"})
        with urllib.request.urlopen(req, timeout=8) as resp:
            html = resp.read().decode("utf-8")
            m = re.search(r'window\.AlgoliaOpts\s*=\s*({[^;]+});', html)
            if m:
                opts = json.loads(m.group(1))
                return opts.get("app", DEFAULT_APP_ID), opts.get("key", DEFAULT_API_KEY)
    except Exception:
        pass
    return DEFAULT_APP_ID, DEFAULT_API_KEY

def fetch_yc_companies(batches: Optional[List[str]] = None, limit: int = 50, query: str = "") -> List[Dict[str, Any]]:
    """Query Algolia for YC companies matching batches and query."""
    app_id, api_key = get_algolia_creds()
    url = f"https://{app_id.lower()}-dsn.algolia.net/1/indexes/*/queries"
    headers = {
        "x-algolia-application-id": app_id,
        "x-algolia-api-key": api_key,
        "content-type": "application/json",
        "user-agent": "Mozilla/5.0"
    }

    if not batches:
        batches = ["Winter 2026", "Winter 2025", "Summer 2025", "Fall 2024", "Summer 2024"]

    # Build facetFilters for batches
    batch_filters = [f"batch:{b}" for b in batches]
    params_str = f"hitsPerPage={limit}&query={urllib.parse.quote(query)}"
    if batch_filters:
        # Algolia facetFilters format: [["batch:Winter 2026", "batch:Winter 2025"]]
        params_str += f"&facetFilters={json.dumps([batch_filters])}"

    body = json.dumps({
        "requests": [
            {
                "indexName": "YCCompany_production",
                "params": params_str
            }
        ]
    }).encode("utf-8")

    req = urllib.request.Request(url, headers=headers, data=body)
    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            return data.get("results", [{}])[0].get("hits", [])
    except Exception as e:
        print(f"[Error fetching YC companies]: {e}", file=sys.stderr)
        return []

def fetch_yc_company_detail(slug: str) -> Dict[str, Any]:
    """Fetch company profile page from YC to extract founders, socials, and bios."""
    url = f"https://www.ycombinator.com/companies/{slug}"
    headers = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"}
    try:
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=8) as resp:
            html = resp.read().decode("utf-8")
            data_pages = re.findall(r'data-page=\"([^\"]+)\"', html)
            if data_pages:
                parsed = json.loads(unescape(data_pages[0]))
                return parsed.get("props", {}).get("company", {})
    except Exception:
        pass
    return {}

def fetch_launch_hn_stories(limit: int = 25) -> List[Dict[str, Any]]:
    """Fetch recent Launch HN posts from Algolia HN search."""
    url = f"https://hn.algolia.com/api/v1/search?query=Launch%20HN&tags=story&hitsPerPage={limit}"
    req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0"})
    try:
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            return data.get("hits", [])
    except Exception as e:
        print(f"[Error fetching Launch HN stories]: {e}", file=sys.stderr)
        return []

if __name__ == "__main__":
    print("[*] Fetching Winter 2026 YC startups...")
    hits = fetch_yc_companies(batches=["Winter 2026"], limit=5)
    print(f"[+] Found {len(hits)} startups")
    for h in hits:
        print(f"- {h.get('name')} ({h.get('batch')}): {h.get('one_liner')} | {h.get('website')}")
        detail = fetch_yc_company_detail(h.get("slug", ""))
        founders = detail.get("founders", [])
        if founders:
            founder_names = ", ".join([f.get("full_name", "") for f in founders if f.get("full_name")])
            twitters = ", ".join([f.get("twitter_url", "") for f in founders if f.get("twitter_url")])
            print(f"  Founders: {founder_names} | Twitter: {twitters}")
