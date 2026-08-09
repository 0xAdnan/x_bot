#!/usr/bin/env python3
import json
import os
import re
import subprocess
from http.server import HTTPServer, SimpleHTTPRequestHandler
import urllib.parse

PORT = 8888
BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
STATE_DIR = os.path.join(BASE_DIR, ".opencode/skills/x-growth/state")

def read_jsonl(filepath):
    items = []
    if os.path.exists(filepath):
        with open(filepath, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        items.append(json.loads(line))
                    except json.JSONDecodeError:
                        pass
    return items

def read_file_text(filepath):
    if os.path.exists(filepath):
        with open(filepath, "r", encoding="utf-8") as f:
            return f.read()
    return ""

def get_budget_stats():
    budget_script = os.path.join(BASE_DIR, ".opencode/skills/x-growth/scripts/budget.sh")
    try:
        res = subprocess.run(["bash", budget_script], capture_output=True, text=True, cwd=BASE_DIR)
        output = res.stdout
    except Exception as e:
        output = str(e)

    cb_script = os.path.join(BASE_DIR, ".opencode/skills/x-growth/scripts/circuit-breaker.sh")
    try:
        cb_res = subprocess.run(["bash", cb_script, "--status"], capture_output=True, text=True, cwd=BASE_DIR)
        cb_status = cb_res.stdout.strip()
    except Exception as e:
        cb_status = str(e)

    account_data = {}
    account_path = os.path.join(STATE_DIR, "account.json")
    if os.path.exists(account_path):
        try:
            with open(account_path, "r") as f:
                account_data = json.load(f)
        except Exception:
            pass

    return {
        "budget_raw": output,
        "circuit_breaker": cb_status,
        "account": account_data
    }

class DashboardHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path == "/api/crm":
            prospects = read_jsonl(os.path.join(STATE_DIR, "prospects.jsonl"))
            stages = {
                "new": [],
                "warming": [],
                "contacted": [],
                "in_convo": [],
                "trial": [],
                "customer": [],
                "do-not-contact": [],
                "lost": []
            }
            for p in prospects:
                st = p.get("stage", "new")
                if st in stages:
                    stages[st].append(p)
                else:
                    stages["new"].append(p)

            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"prospects": prospects, "stages": stages, "total": len(prospects)}).encode("utf-8"))
            return

        elif path == "/api/activity":
            activities = read_jsonl(os.path.join(STATE_DIR, "activity-log.jsonl"))
            activities.reverse()  # Newest first
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"activities": activities[:100], "total": len(activities)}).encode("utf-8"))
            return

        elif path == "/api/stats":
            stats = get_budget_stats()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(stats).encode("utf-8"))
            return

        elif path == "/api/insights":
            text = read_file_text(os.path.join(STATE_DIR, "insights.md"))
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"content": text}).encode("utf-8"))
            return

        elif path == "/" or path == "/index.html":
            html_path = os.path.join(os.path.dirname(__file__), "index.html")
            if os.path.exists(html_path):
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.end_headers()
                with open(html_path, "rb") as f:
                    self.wfile.write(f.read())
                return

        return super().do_GET()

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path == "/api/prospect/add":
            length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(length)
            try:
                data = json.loads(body)
                prospect = {
                    "handle": data.get("handle", ""),
                    "name": data.get("name", ""),
                    "url": data.get("url", ""),
                    "segment": data.get("segment", "founder"),
                    "score": int(data.get("score", 0)),
                    "stage": data.get("stage", "new"),
                    "last_touch": "",
                    "next_action_date": "",
                    "touches": 0,
                    "product_url": data.get("product_url", ""),
                    "last_variant": "",
                    "outcome": "",
                    "notes": data.get("notes", ""),
                    "why": data.get("why", "")
                }
                filepath = os.path.join(STATE_DIR, "prospects.jsonl")
                with open(filepath, "a", encoding="utf-8") as f:
                    f.write(json.dumps(prospect) + "\n")
                
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({"success": True, "prospect": prospect}).encode("utf-8"))
                return
            except Exception as e:
                self.send_response(400)
                self.end_headers()
                self.wfile.write(json.dumps({"error": str(e)}).encode("utf-8"))
                return

        self.send_response(404)
        self.end_headers()

if __name__ == "__main__":
    os.makedirs(STATE_DIR, exist_ok=True)
    print(f"Starting X Growth CRM Dashboard on http://0.0.0.0:{PORT}")
    server = HTTPServer(("0.0.0.0", PORT), DashboardHandler)
    server.serve_forever()
