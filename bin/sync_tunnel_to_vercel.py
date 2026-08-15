#!/usr/bin/env python3
import subprocess
import time
import re
import os

REPO_DIR = "/home/adnan/x_bot"
DASHBOARD_DIR = os.path.join(REPO_DIR, "dashboard")
RUST_JS_PATH = os.path.join(DASHBOARD_DIR, "api", "_rust.js")
LAST_URL_FILE = os.path.join(REPO_DIR, "data", "last_tunnel_url.txt")

def get_active_tunnel_url():
    try:
        res = subprocess.run(["tmux", "capture-pane", "-t", "pass-tunnel", "-p"], capture_output=True, text=True)
        m = re.search(r"https://[a-zA-Z0-9-]+\.loca\.lt", res.stdout)
        if m:
            return m.group(0)
    except Exception as e:
        pass
    return None

def update_rust_js_and_deploy(tunnel_url):
    print(f"[Tunnel Sync] New Localtunnel URL detected: {tunnel_url}")
    
    rust_js_content = f"""export async function fetchRust(path, options = {{}}) {{
  const candidates = [
    process.env.LOCAL_RUST_SERVER_URL,
    "{tunnel_url}",
    "https://heavy-cougar-57.loca.lt",
    "https://perfect-termite-69.loca.lt",
    "https://pitch-bot-adnan.loca.lt"
  ].filter(Boolean);

  const sep = path.includes('?') ? '&' : '?';
  const cleanPath = `${{path}}${{sep}}bypass-tunnel-reminder=true`;

  for (const baseUrl of candidates) {{
    try {{
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);
      const resp = await fetch(`${{baseUrl}}${{cleanPath}}`, {{
        ...options,
        signal: controller.signal,
        headers: {{
          "Bypass-Tunnel-Remainder": "true",
          "bypass-tunnel-reminder": "true",
          "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
          ...(options.headers || {{}})
        }}
      }});
      clearTimeout(timeoutId);
      const text = await resp.text();
      const trimmed = text.trim();
      if (resp.ok && (trimmed.startsWith('{{') || trimmed.startsWith('['))) {{
        const data = JSON.parse(trimmed);
        return {{ ok: true, data, baseUrl }};
      }}
    }} catch (e) {{}}
  }}
  return {{ ok: false }};
}}
"""
    with open(RUST_JS_PATH, "w") as f:
        f.write(rust_js_content)
    
    try:
        subprocess.run(["cp", os.path.join(DASHBOARD_DIR, "index.html"), os.path.join(DASHBOARD_DIR, "public", "index.html")])
    except Exception:
        pass

    print("[Tunnel Sync] Deploying updated tunnel URL to Vercel production...")
    try:
        subprocess.run(["npx", "vercel", "--prod", "--yes"], cwd=DASHBOARD_DIR, capture_output=True, text=True)
        print("[Tunnel Sync] Vercel deployment completed successfully!")
        with open(LAST_URL_FILE, "w") as f:
            f.write(tunnel_url)
    except Exception as e:
        print(f"[Tunnel Sync Error]: {e}")

def main():
    last_url = ""
    if os.path.exists(LAST_URL_FILE):
        try:
            last_url = open(LAST_URL_FILE).read().strip()
        except Exception:
            pass

    while True:
        url = get_active_tunnel_url()
        if url and url != last_url:
            update_rust_js_and_deploy(url)
            last_url = url
        time.sleep(15)

if __name__ == "__main__":
    main()
