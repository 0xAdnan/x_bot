#!/usr/bin/env python3
"""
Outreach Dispatch & Review Utility
Views pending drafted outreach, exports ready-to-send campaigns,
and optionally sends emails via SMTP (when configured in .env).
"""

import sys
import os
import smtplib
import sqlite3
import argparse
import json
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from pathlib import Path
from typing import List, Dict, Any, Optional

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[3]
DB_PATH = Path(os.environ.get("SQLITE_DB_PATH", REPO_ROOT / "data" / "pitch_bot.db"))

def load_env_file():
    env_file = REPO_ROOT / ".env"
    if env_file.exists():
        with open(env_file, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    k = k.strip()
                    v = v.strip().strip('"').strip("'")
                    if k not in os.environ:
                        os.environ[k] = v

load_env_file()

def get_db_connection() -> sqlite3.Connection:
    conn = sqlite3.connect(str(DB_PATH))
    conn.row_factory = sqlite3.Row
    return conn

def list_researched_startups(limit: int = 20) -> List[sqlite3.Row]:
    conn = get_db_connection()
    cur = conn.execute(
        "SELECT id, handle, name, segment, score, stage, product_url, email, batch, notes, why FROM prospects WHERE stage IN ('researched', 'new') AND segment IN ('yc', 'antler') ORDER BY id DESC LIMIT ?",
        (limit,)
    )
    return cur.fetchall()

def send_email_smtp(to_email: str, subject: str, body: str) -> bool:
    smtp_host = os.environ.get("SMTP_HOST")
    smtp_port = int(os.environ.get("SMTP_PORT", 587))
    smtp_user = os.environ.get("SMTP_USER")
    smtp_pass = os.environ.get("SMTP_PASS")
    from_email = os.environ.get("SMTP_FROM", smtp_user or "adnan@trypitch.co")

    if not smtp_host or not smtp_user or not smtp_pass:
        print("[-] SMTP credentials not configured in .env (SMTP_HOST, SMTP_USER, SMTP_PASS required).")
        return False

    # Clean body: strip DM or Tweet sections if present
    clean_body = body
    if "Email Body:\n" in clean_body:
        clean_body = clean_body.split("Email Body:\n")[-1]
    if "\n\nX DM" in clean_body:
        clean_body = clean_body.split("\n\nX DM")[0]
    clean_body = clean_body.strip()

    # Ensure signature block is always attached
    if "adnan@trypitch.co" not in clean_body:
        clean_body = f"{clean_body}\n\nBest,\nAdnan\nCo-Founder, Pitch\nadnan@trypitch.co"

    try:
        msg = MIMEMultipart()
        msg["From"] = from_email
        msg["To"] = to_email
        msg["Subject"] = subject
        msg.attach(MIMEText(clean_body, "plain"))

        with smtplib.SMTP(smtp_host, smtp_port) as server:
            server.starttls()
            server.login(smtp_user, smtp_pass)
            server.send_message(msg)

        print(f"[+] Email successfully sent to {to_email}")
        return True
    except Exception as e:
        print(f"[-] Failed to send email to {to_email}: {e}")
        return False

def check_already_contacted(to_email: str, handle: str) -> bool:
    """Safety guardrail: prevent duplicate cold emails to the same recipient."""
    conn = get_db_connection()
    clean_h = handle.strip() if handle else ""
    cur = conn.execute(
        "SELECT id, stage, touches, last_touch FROM prospects WHERE (handle = ? AND handle != '') OR (email = ? AND email != '')",
        (clean_h, to_email.strip())
    )
    row = cur.fetchone()
    if row and (row["stage"] == "contacted" or row["stage"] == "in_convo" or (row["touches"] and row["touches"] > 0)):
        return True
    return False

def mark_prospect_contacted(handle: str, channel: str = "email"):
    conn = get_db_connection()
    conn.execute(
        "UPDATE prospects SET stage = 'contacted', last_touch = DATE('now'), touches = touches + 1, updated_at = CURRENT_TIMESTAMP WHERE handle = ?",
        (handle,)
    )
    conn.execute(
        "INSERT INTO activities (ts, action, handle, segment, variant, detail, result) VALUES (DATETIME('now'), ?, ?, 'startup_outreach', 'v1', ?, 'ok')",
        (f"outreach_{channel}", handle, f"Outreach sent via {channel}")
    )
    conn.commit()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Startup Outreach Review & Dispatch")
    parser.add_argument("--list", action="store_true", help="List researched startups ready for outreach")
    parser.add_argument("--limit", type=int, default=10, help="Number of records to show")
    parser.add_argument("--send-email", help="Handle of prospect to send email to")
    parser.add_argument("--send-now", action="store_true", help="Send email directly via CLI")
    parser.add_argument("--to", help="Recipient email address")
    parser.add_argument("--subject", help="Email subject line")
    parser.add_argument("--body", help="Email body text")
    parser.add_argument("--handle", help="Prospect handle to mark contacted")
    parser.add_argument("--mark-contacted", help="Handle of prospect to mark contacted")
    parser.add_argument("--force", action="store_true", help="Force send even if already contacted")
    parser.add_argument("--export-csv", help="Export researched prospects to CSV")

    args = parser.parse_args()

    if args.send_now and args.to:
        if not args.force and args.to != "adnan.pitch@gmail.com" and check_already_contacted(args.to, args.handle):
            print(json.dumps({
                "status": "skipped",
                "message": f"Prospect {args.to} ({args.handle}) was already contacted. Single-send safety enforced.",
                "to_email": args.to
            }))
            sys.exit(0)

        subject = args.subject or "quick demo video for your startup"
        body = args.body or ""
        ok = send_email_smtp(args.to, subject, body)
        if ok:
            if args.handle:
                mark_prospect_contacted(args.handle, channel="email")
            print(json.dumps({"status": "ok", "message": f"Email successfully sent to {args.to}", "to_email": args.to}))
            sys.exit(0)
        else:
            print(json.dumps({"status": "error", "message": f"Failed to send email to {args.to}"}))
            sys.exit(1)

    if args.list or len(sys.argv) == 1:
        prospects = list_researched_startups(limit=args.limit)
        print(f"=== [STARTUPS READY FOR OUTREACH] ({len(prospects)} items) ===")
        for p in prospects:
            print(f"\n[{p['id']}] {p['name']} ({p['handle']}) | Segment: {p['segment']} | Batch: {p['batch']}")
            print(f"    Website: {p['product_url']} | Email: {p['email'] or 'N/A'}")
            print(f"    Notes/Draft:\n{p['notes']}")
            print("-" * 60)

    if args.mark_contacted:
        mark_prospect_contacted(args.mark_contacted)
        print(f"[+] Marked {args.mark_contacted} as contacted.")

    if args.export_csv:
        import csv
        prospects = list_researched_startups(limit=100)
        with open(args.export_csv, "w", newline="", encoding="utf-8") as f:
            writer = csv.writer(f)
            writer.writerow(["ID", "Name", "Handle", "Email", "Batch", "Segment", "Website", "Notes", "Stage"])
            for p in prospects:
                writer.writerow([p["id"], p["name"], p["handle"], p["email"], p["batch"], p["segment"], p["product_url"], p["notes"], p["stage"]])
        print(f"[+] Exported {len(prospects)} prospects to {args.export_csv}")
