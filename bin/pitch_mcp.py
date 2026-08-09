#!/usr/bin/env python3
import urllib.request
import json
import sys
import time

PITCH_API_KEY = "pk_tltxrmrZgiprXR51z_dJvoIF0yWiGBVB"
PITCH_MCP_URL = "https://api.trypitch.co/mcp"

def call_mcp_tool(tool_name, arguments):
    payload = {
        "jsonrpc": "2.0",
        "id": int(time.time()),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    }
    
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(PITCH_MCP_URL, data=data, headers={
        "Authorization": f"Bearer {PITCH_API_KEY}",
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
        "User-Agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    })
    
    try:
        with urllib.request.urlopen(req) as resp:
            text = resp.read().decode("utf-8")
            for line in text.split("\n"):
                if line.startswith("data: "):
                    parsed = json.loads(line[6:])
                    content = parsed.get("result", {}).get("content", [])
                    if content and content[0].get("type") == "text":
                        return json.loads(content[0].get("text"))
            return None
    except Exception as e:
        print(f"[Pitch MCP Error] {e}", file=sys.stderr)
        return None

def create_video_job(url, instructions=None):
    if not instructions:
        instructions = f"Create a cinematic, polished product demo of {url}. Highlight key features, value proposition, and user experience."
    
    arguments = {
        "url": url,
        "instructions": instructions,
        "voice": "Charon",
        "subtitles": False,
        "theme": "light",
        "background": "ocean",
        "shape": "rounded",
        "inset": "0.75",
        "browserHeader": "light"
    }
    return call_mcp_tool("create_demo_video", arguments)

def get_job_status(job_id):
    return call_mcp_tool("get_job", {"jobId": job_id})

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: pitch_mcp.py <create|status|credits> [args...]")
        sys.exit(1)
        
    cmd = sys.argv[1]
    if cmd == "create":
        target_url = sys.argv[2]
        custom_inst = sys.argv[3] if len(sys.argv) > 3 else None
        res = create_video_job(target_url, custom_inst)
        print(json.dumps(res, indent=2))
    elif cmd == "status":
        job_id = sys.argv[2]
        res = get_job_status(job_id)
        print(json.dumps(res, indent=2))
    elif cmd == "credits":
        res = call_mcp_tool("get_credits", {})
        print(json.dumps(res, indent=2))
