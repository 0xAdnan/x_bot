#!/usr/bin/env python3
import urllib.request
import urllib.parse
import urllib.error
import json
import base64
import os
import sys

ENV_PATH = "/home/adnan/x_bot/.env"

def try_b64decode(s):
    if not s:
        return s
    for pad in ['', '=', '==', '===']:
        try:
            dec = base64.b64decode(s + pad).decode('utf-8')
            if dec and all(32 <= ord(c) < 127 for c in dec) and dec != s:
                return dec
        except Exception:
            pass
    return s

def load_env():
    env = {}
    if os.path.exists(ENV_PATH):
        with open(ENV_PATH) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    k, v = line.split('=', 1)
                    env[k.strip()] = v.strip().strip('\"\'')
    return env

def update_env(new_access, new_refresh):
    if not os.path.exists(ENV_PATH):
        return
    with open(ENV_PATH, 'r') as f:
        lines = f.readlines()
    with open(ENV_PATH, 'w') as f:
        for line in lines:
            if line.startswith('X_USER_ACCESS_TOKEN='):
                f.write(f"X_USER_ACCESS_TOKEN={new_access}\n")
            elif line.startswith('X_USER_REFRESH_TOKEN='):
                f.write(f"X_USER_REFRESH_TOKEN={new_refresh}\n")
            else:
                f.write(line)
    print("[Token Refresher] Successfully updated .env with new X_USER_ACCESS_TOKEN and X_USER_REFRESH_TOKEN")

def do_refresh():
    env = load_env()
    raw_cid = env.get("X_CLIENT_ID")
    raw_sec = env.get("X_CLIENT_SECRET")
    refresh_token = env.get("X_USER_REFRESH_TOKEN")

    client_id = try_b64decode(raw_cid)
    client_secret = try_b64decode(raw_sec)

    if not client_id or not refresh_token:
        print("[Token Refresher Error] Missing X_CLIENT_ID or X_USER_REFRESH_TOKEN in .env")
        return False

    url = "https://api.twitter.com/2/oauth2/token"
    payload = {
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": client_id
    }
    data = urllib.parse.urlencode(payload).encode("utf-8")

    headers = {
        "Content-Type": "application/x-www-form-urlencoded"
    }

    if client_secret:
        auth_str = f"{client_id}:{client_secret}"
        b64_auth = base64.b64encode(auth_str.encode("utf-8")).decode("utf-8")
        headers["Authorization"] = f"Basic {b64_auth}"

    req = urllib.request.Request(url, data=data, headers=headers, method="POST")

    try:
        with urllib.request.urlopen(req) as resp:
            body = json.loads(resp.read().decode("utf-8"))
            new_access = body.get("access_token")
            new_refresh = body.get("refresh_token") or refresh_token
            if new_access:
                update_env(new_access, new_refresh)
                return True
            else:
                print(f"[Token Refresher Error] No access_token in response: {body}")
    except urllib.error.HTTPError as e:
        print(f"[Token Refresher HTTP Error]: {e.read().decode('utf-8')}")
    except Exception as e:
        print(f"[Token Refresher Error]: {e}")
    return False

if __name__ == "__main__":
    do_refresh()
