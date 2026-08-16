#!/usr/bin/env python3
"""
Master YC & Antler Startup Scout, Enricher, and Outreach Engine
Integrates Algolia YC search, Antler Webflow portfolio scraping,
website contact enrichment, anti-AI copy generation, and SQLite CRM syncing.
"""

import sys
import os
import re
import argparse
import json
import sqlite3
from pathlib import Path
from typing import List, Dict, Any, Optional

# Ensure scripts dir is on sys.path
SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[3]
sys.path.insert(0, str(SCRIPT_DIR))

from scrape_yc import fetch_yc_companies, fetch_yc_company_detail
from scrape_antler import fetch_antler_companies
from contact_enricher import enrich_startup
from draft_generator import generate_outreach_drafts, validate_draft

DB_PATH = Path(os.environ.get("SQLITE_DB_PATH", REPO_ROOT / "data" / "pitch_bot.db"))
PROSPECTS_JSONL = REPO_ROOT / ".opencode" / "skills" / "x-growth" / "state" / "prospects.jsonl"
ACTIVITY_LOG_JSONL = REPO_ROOT / ".opencode" / "skills" / "x-growth" / "state" / "activity-log.jsonl"

def get_db_connection() -> sqlite3.Connection:
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    # Ensure tables and columns exist
    conn.execute("""
        CREATE TABLE IF NOT EXISTS prospects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            handle TEXT UNIQUE NOT NULL,
            name TEXT DEFAULT '',
            url TEXT DEFAULT '',
            segment TEXT DEFAULT 'founder',
            score INTEGER DEFAULT 0,
            stage TEXT DEFAULT 'new',
            last_touch TEXT,
            next_action_date TEXT,
            touches INTEGER DEFAULT 0,
            product_url TEXT DEFAULT '',
            last_variant TEXT DEFAULT '',
            outcome TEXT DEFAULT '',
            notes TEXT DEFAULT '',
            why TEXT DEFAULT '',
            email TEXT DEFAULT '',
            batch TEXT DEFAULT '',
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );
    """)
    try:
        conn.execute("ALTER TABLE prospects ADD COLUMN email TEXT DEFAULT '';")
    except Exception:
        pass
    try:
        conn.execute("ALTER TABLE prospects ADD COLUMN batch TEXT DEFAULT '';")
    except Exception:
        pass
    return conn

def is_already_in_db(conn: sqlite3.Connection, handle: str, website: str) -> bool:
    """Check if handle or website already exists in prospects table."""
    clean_h = handle.lower().strip()
    if clean_h and clean_h != "n/a" and clean_h != "@prospect":
        cur = conn.execute("SELECT id FROM prospects WHERE LOWER(handle) = ?", (clean_h,))
        if cur.fetchone():
            return True
            
    if website and website != "N/A" and len(website) > 8:
        clean_w = website.replace("https://", "").replace("http://", "").replace("www.", "").rstrip("/").lower()
        cur = conn.execute("SELECT id FROM prospects WHERE LOWER(product_url) LIKE ?", (f"%{clean_w}%",))
        if cur.fetchone():
            return True
            
    return False

def save_prospect_to_db(conn: sqlite3.Connection, p: Dict[str, Any]):
    """Insert or update prospect in SQLite and JSONL."""
    handle = p.get("handle") or f"@{p.get('name', 'startup').lower().replace(' ', '')}"
    conn.execute("""
        INSERT INTO prospects (handle, name, url, segment, score, stage, last_touch, next_action_date, touches, product_url, last_variant, outcome, notes, why, email, batch, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        ON CONFLICT(handle) DO UPDATE SET
            name = COALESCE(NULLIF(excluded.name, ''), prospects.name),
            url = COALESCE(NULLIF(excluded.url, ''), prospects.url),
            segment = COALESCE(NULLIF(excluded.segment, ''), prospects.segment),
            score = COALESCE(excluded.score, prospects.score),
            stage = COALESCE(NULLIF(excluded.stage, ''), prospects.stage),
            product_url = COALESCE(NULLIF(excluded.product_url, ''), prospects.product_url),
            notes = COALESCE(NULLIF(excluded.notes, ''), prospects.notes),
            why = COALESCE(NULLIF(excluded.why, ''), prospects.why),
            email = COALESCE(NULLIF(excluded.email, ''), prospects.email),
            batch = COALESCE(NULLIF(excluded.batch, ''), prospects.batch),
            updated_at = CURRENT_TIMESTAMP
    """, (
        handle,
        p.get("name", ""),
        p.get("url", ""),
        p.get("segment", "founder"),
        p.get("score", 9),
        p.get("stage", "new"),
        p.get("last_touch"),
        p.get("next_action_date"),
        p.get("touches", 0),
        p.get("product_url", ""),
        p.get("last_variant", ""),
        p.get("outcome", ""),
        p.get("notes", ""),
        p.get("why", ""),
        p.get("email", ""),
        p.get("batch", "")
    ))
    conn.commit()

    # Mirror to JSONL
    mirror_to_jsonl(p)

def mirror_to_jsonl(p: Dict[str, Any]):
    PROSPECTS_JSONL.parent.mkdir(parents=True, exist_ok=True)
    records = []
    if PROSPECTS_JSONL.exists():
        with open(PROSPECTS_JSONL, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        records.append(json.loads(line))
                    except Exception:
                        pass
    
    handle = p.get("handle")
    updated = False
    for idx, rec in enumerate(records):
        if rec.get("handle", "").lower() == handle.lower():
            records[idx].update(p)
            updated = True
            break
    if not updated:
        records.append(p)
        
    with open(PROSPECTS_JSONL, "w", encoding="utf-8") as f:
        for r in records:
            f.write(json.dumps(r) + "\n")

def run_scout(source: str = "all", batches: Optional[List[str]] = None, limit: int = 10, dry_run: bool = False) -> List[Dict[str, Any]]:
    print(f"=== [STARTUP SCOUT: Source={source.upper()} | Limit={limit} | DryRun={dry_run}] ===")
    conn = get_db_connection()
    raw_startups = []

    if source in ["yc", "all"]:
        yc_batches = batches or ["Winter 2026", "Winter 2025", "Summer 2025"]
        print(f"[*] Querying YC directory for batches: {yc_batches}...")
        yc_hits = fetch_yc_companies(batches=yc_batches, limit=limit)
        print(f"[+] Retrieved {len(yc_hits)} YC startups")
        for hit in yc_hits:
            hit["source"] = "yc"
            raw_startups.append(hit)

    if source in ["antler", "all"]:
        print(f"[*] Querying Antler Global Portfolio...")
        antler_hits = fetch_antler_companies(limit=limit)
        print(f"[+] Retrieved {len(antler_hits)} Antler startups")
        for hit in antler_hits:
            hit["source"] = "antler"
            raw_startups.append(hit)

    results = []
    saved_count = 0

    for idx, s in enumerate(raw_startups[:limit]):
        name = s.get("name", "Unknown")
        website = s.get("website", "")
        batch = s.get("batch", "Upcoming")
        source_type = s.get("source", "yc")
        
        print(f"\n[{idx+1}/{min(len(raw_startups), limit)}] Processing {name} ({batch})...")

        # Founder details from YC profile if available
        founders = []
        if source_type == "yc" and s.get("slug"):
            detail = fetch_yc_company_detail(s.get("slug"))
            founders = detail.get("founders", [])

        # Enrich with website contact info & metadata
        enrich_data = enrich_startup(website, {
            "name": name,
            "batch": batch,
            "one_liner": s.get("one_liner") or s.get("description", "")
        })

        # Determine primary handle and email
        primary_handle = ""
        # 1. From YC founder profile
        for f in founders:
            t_url = f.get("twitter_url", "")
            if t_url:
                handle_m = re.search(r'(?:twitter\.com|x\.com)/([a-zA-Z0-9_]+)', t_url)
                if handle_m:
                    primary_handle = f"@{handle_m.group(1)}"
                    break
        # 2. From website enricher
        if not primary_handle and enrich_data.get("primary_handle"):
            primary_handle = enrich_data["primary_handle"]
        # 3. Fallback handle
        if not primary_handle:
            clean_name = re.sub(r'[^a-zA-Z0-9_]', '', name.lower())
            primary_handle = f"@{clean_name}"

        primary_email = enrich_data.get("primary_email", "")

        # Check DB deduplication
        if is_already_in_db(conn, primary_handle, website):
            print(f"    [ALREADY IN CRM]: Skipping {name} ({primary_handle})")
            continue

        # Generate Human Anti-AI Outreach Drafts
        startup_payload = {
            "name": name,
            "founders": founders,
            "batch": batch,
            "website": website,
            "one_liner": s.get("one_liner") or s.get("description", ""),
            "primary_handle": primary_handle
        }
        drafts = generate_outreach_drafts(startup_payload)

        # Validate anti-AI rules
        if drafts["validation_email"] or drafts["validation_dm"]:
            print(f"    [WARNING] Anti-AI validation warnings: {drafts['validation_email']} {drafts['validation_dm']}")

        prospect_record = {
            "handle": primary_handle,
            "name": founders[0].get("full_name") if founders else name,
            "url": f"https://x.com/{primary_handle.replace('@', '')}" if primary_handle.startswith("@") else website,
            "segment": source_type,
            "score": 9,
            "stage": "researched",
            "touches": 0,
            "product_url": website,
            "email": primary_email,
            "batch": batch,
            "notes": f"Email Subj: {drafts['email_subject']}\nEmail Body:\n{drafts['email_body']}\n\nX DM:\n{drafts['x_dm']}",
            "why": f"Upcoming {batch} batch startup. Product: {startup_payload['one_liner']}"
        }

        print(f"    + Discovered: {name}")
        print(f"      Handle: {primary_handle} | Email: {primary_email or 'N/A'}")
        print(f"      Product: {startup_payload['one_liner']}")
        print(f"      Email Subject: {drafts['email_subject']}")
        print(f"      X DM: {drafts['x_dm']}")

        if not dry_run:
            save_prospect_to_db(conn, prospect_record)
            saved_count += 1

        results.append({
            "startup": startup_payload,
            "enrichment": enrich_data,
            "drafts": drafts,
            "prospect_record": prospect_record
        })

    print(f"\n=== [SCOUT COMPLETE] Processed: {len(results)} | Saved to CRM: {saved_count} ===")
    return results

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="YC & Antler Startup Scout & Outreach Engine")
    parser.add_argument("--source", choices=["yc", "antler", "all"], default="all", help="Source to scout")
    parser.add_argument("--batch", action="append", help="YC batch to filter (e.g. 'Winter 2026')")
    parser.add_argument("--limit", type=int, default=5, help="Max startups to process")
    parser.add_argument("--dry-run", action="store_true", help="Perform dry run without saving to DB")
    parser.add_argument("--output-json", help="Path to write JSON results")

    args = parser.parse_args()
    res = run_scout(source=args.source, batches=args.batch, limit=args.limit, dry_run=args.dry_run)

    if args.output_json:
        with open(args.output_json, "w", encoding="utf-8") as f:
            json.dump(res, f, indent=2)
        print(f"[+] Wrote results to {args.output_json}")
