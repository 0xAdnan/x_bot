#!/usr/bin/env python3
"""
Antler Global Startup Scout & Contact Extractor
Fetches startups from Antler Global Portfolio (https://www.antler.co/portfolio)
including location, sector, year, website, and description.
"""

import sys
import os
import re
import json
import urllib.request
from html import unescape
from typing import List, Dict, Any

ANTLER_PORTFOLIO_URL = "https://www.antler.co/portfolio"

def fetch_antler_companies(limit: int = 50, location_filter: str = "") -> List[Dict[str, Any]]:
    """Scrape Antler portfolio from antler.co/portfolio HTML structure."""
    headers = {"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"}
    req = urllib.request.Request(ANTLER_PORTFOLIO_URL, headers=headers)
    companies = []

    try:
        with urllib.request.urlopen(req, timeout=12) as resp:
            html = resp.read().decode("utf-8")
            
            # Antler uses Webflow collection cards with portco_card
            # Each card has name, description, location, sector, year, and a link
            cards = re.findall(r'<div[^>]*class=\"[^\"]*portco_card[^\"]*\"[^>]*>(.*?)</div>\s*</div>\s*</div>', html, re.DOTALL)
            if not cards:
                # Fallback: search for cards by headings
                cards = re.findall(r'<div[^>]*class=\"portco_card_content\"[^>]*>(.*?)</div>\s*</div>', html, re.DOTALL)

            # Let's extract items using regex patterns
            # Find all name paragraphs
            name_blocks = re.findall(
                r'<p[^>]*fs-cmsfilter-field=\"name\"[^>]*>(.*?)</p>\s*<p[^>]*fs-cmsfilter-field=\"description\"[^>]*>(.*?)</p>',
                html,
                re.DOTALL
            )
            
            # Find associated links
            card_chunks = html.split('class="portco_card"')
            for chunk in card_chunks[1:]:
                name_match = re.search(r'fs-cmsfilter-field=\"name\"[^>]*>([^<]+)</p>', chunk)
                desc_match = re.search(r'fs-cmsfilter-field=\"description\"[^>]*>([^<]+)</p>', chunk)
                loc_match = re.search(r'fs-cmsfilter-field=\"location\"[^>]*>([^<]+)</div>', chunk)
                sector_match = re.search(r'fs-cmsfilter-field=\"sector\"[^>]*>.*?<div[^>]*>([^<]+)</div>', chunk, re.DOTALL)
                year_match = re.search(r'fs-cmsfilter-field=\"year\"[^>]*>.*?<div[^>]*>([^<]+)</div>', chunk, re.DOTALL)
                links = re.findall(r'href=\"(https?://[^\"]+)\"', chunk)
                link = ""
                for l in links:
                    if "antler.co" not in l and not l.endswith(".png") and not l.endswith(".svg") and not l.endswith(".avif"):
                        link = l
                        break

                if name_match:
                    name = unescape(name_match.group(1).strip())
                    desc = unescape(desc_match.group(1).strip()) if desc_match else ""
                    loc = unescape(loc_match.group(1).strip()) if loc_match else "Global"
                    sector = unescape(sector_match.group(1).strip()) if sector_match else "Tech"
                    year = unescape(year_match.group(1).strip()) if year_match else "2024"

                    if location_filter and location_filter.lower() not in loc.lower():
                        continue

                    companies.append({
                        "name": name,
                        "description": desc,
                        "location": loc,
                        "sector": sector,
                        "year": year,
                        "website": link,
                        "batch": f"Antler {loc} {year}",
                        "source": "antler"
                    })

                    if len(companies) >= limit:
                        break

    except Exception as e:
        print(f"[Error fetching Antler portfolio]: {e}", file=sys.stderr)

    return companies

if __name__ == "__main__":
    print("[*] Fetching Antler Global startups...")
    startups = fetch_antler_companies(limit=5)
    print(f"[+] Found {len(startups)} Antler startups")
    for s in startups:
        print(f"- {s['name']} ({s['location']}, {s['year']}): {s['description']} | {s['website']}")
