#!/usr/bin/env bash
# budget.sh — today's remaining X-action budget from state/activity-log.jsonl
#
# The agent runs this at the start of every session (and before any action type
# it's unsure about) to stay under the daily caps in safety.md.
#
# Usage:
#   bash .opencode/skills/x-growth/scripts/budget.sh            # today (local date)
#   bash .opencode/skills/x-growth/scripts/budget.sh 2026-08-09 # a specific day
#   bash .opencode/skills/x-growth/scripts/budget.sh --json     # machine-readable
#
# Only actions with "result":"ok" count toward caps. The DM cap covers both
# "dm" and "followup". Caps below mirror safety.md — keep them in sync.
#
# Cold-start ramp (anti-ban, enforced automatically): if state/account.json has a
# `ramp_until` date in the future, caps are multiplied by COLD_FACTOR (25%).
# This is how a brand-new account avoids the volume that got @adnanxpitch banned
# in 6 days. An explicit env var (e.g. CAP_DM=3) beats the ramp for that run;
# otherwise the ramp applies.
#
# Burst check: also reports actions in the last 60 minutes so the agent can avoid
# back-to-back bursts (a bot fingerprint).
set -euo pipefail

# --- caps (mirror safety.md) -------------------------------------------------
# NOTE: do NOT pre-default CAP_* here. capval() below reads the real environment
# to tell an intentional override (CAP_DM=3 bash budget.sh) apart from a default,
# so the cold-start ramp can apply to defaults. Only COLD_FACTOR/BURST_HR are
# pre-defaulted here (they are not per-action caps).
COLD_FACTOR="${COLD_FACTOR:-25}"   # percent of caps during the ramp window
BURST_HR="${BURST_HR:-10}"         # max ok actions per rolling hour

# --- locate the log relative to this script ----------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../" && pwd)"             # scripts/ -> skill root (contains state/)
LOG="$ROOT/state/activity-log.jsonl"
ACCOUNT="$ROOT/state/account.json"

DAY=""; JSON=0
for arg in "$@"; do
  case "$arg" in
    --json) JSON=1 ;;
    *) DAY="$arg" ;;
  esac
done
[ -z "$DAY" ] && DAY="$(date +%F)"

if [ ! -f "$LOG" ]; then
  echo "No log at $LOG — assuming a fresh day (full budget)." >&2
  LOG="/dev/null"
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "budget.sh needs python3." >&2; exit 1
fi

# --- cold-start ramp ----------------------------------------------------------
RAMP_UNTIL=""
if [ -f "$ACCOUNT" ]; then
  RAMP_UNTIL=$(python3 -c "import json;print(json.load(open('$ACCOUNT')).get('ramp_until','') or '')" 2>/dev/null || true)
fi
RAMP=0
if [ -n "$RAMP_UNTIL" ]; then
  IS_RAMP=$(python3 - "$DAY" "$RAMP_UNTIL" <<'PY'
import sys
print(1 if sys.argv[1] < sys.argv[2] else 0)
PY
)
  [ "$IS_RAMP" = "1" ] && RAMP=1
fi

# Apply the cold factor to the DEFAULT caps only. An explicitly-set env var is an
# intentional override and always wins.
capval() { # name default
  local name="$1" default="$2"
  local var="CAP_$name"
  local val
  if [ -n "${!var+x}" ]; then
    val="${!var}"
  else
    val="$default"
    if [ "$RAMP" = "1" ]; then
      val=$(( (val * COLD_FACTOR) / 100 ))
      [ "$val" -lt 1 ] && val=1
    fi
  fi
  echo "$val"
}

CAP_LIKE=$(capval LIKE 50)
CAP_REPLY=$(capval REPLY 15)
CAP_FOLLOW=$(capval FOLLOW 15)
CAP_DM=$(capval DM 10)
CAP_POST=$(capval POST 4)
CAP_QUOTE=$(capval QUOTE 4)
CAP_DISCOVER=$(capval DISCOVER 40)

DAY="$DAY" JSON="$JSON" RAMP="$RAMP" RAMP_UNTIL="$RAMP_UNTIL" \
CAP_LIKE="$CAP_LIKE" CAP_REPLY="$CAP_REPLY" CAP_FOLLOW="$CAP_FOLLOW" \
CAP_DM="$CAP_DM" CAP_POST="$CAP_POST" CAP_QUOTE="$CAP_QUOTE" \
CAP_DISCOVER="$CAP_DISCOVER" BURST_HR="$BURST_HR" \
python3 - "$LOG" <<'PY'
import json, os, sys, time

log = sys.argv[1]
day = os.environ["DAY"]
as_json = os.environ["JSON"] == "1"
ramp = os.environ["RAMP"] == "1"
ramp_until = os.environ.get("RAMP_UNTIL", "")
burst_hr = int(os.environ["BURST_HR"])
caps = {
    "like": int(os.environ["CAP_LIKE"]),
    "reply": int(os.environ["CAP_REPLY"]),
    "follow": int(os.environ["CAP_FOLLOW"]),
    "dm": int(os.environ["CAP_DM"]),          # dm + followup
    "post": int(os.environ["CAP_POST"]),
    "quote": int(os.environ["CAP_QUOTE"]),
    "discover": int(os.environ["CAP_DISCOVER"]),
}
used = {k: 0 for k in caps}
last_hour = 0
now_epoch = time.time()

def parse_epoch(ts):
    try:
        from datetime import datetime, timezone
        s = str(ts)
        if s.endswith("Z"):
            s = s[:-1] + "+00:00"
        return datetime.fromisoformat(s).timestamp()
    except Exception:
        return None

try:
    with open(log) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            if str(e.get("ts", "")).split("T")[0] != day:
                continue
            if e.get("result") != "ok":
                continue
            a = e.get("action")
            if a in ("dm", "followup"):
                used["dm"] += 1
            elif a in used:
                used[a] += 1
            ep = parse_epoch(e.get("ts", ""))
            if ep is not None and now_epoch - ep <= 3600:
                last_hour += 1
except FileNotFoundError:
    pass

remaining = {k: max(0, caps[k] - used[k]) for k in caps}
burst = last_hour >= burst_hr

if as_json:
    print(json.dumps({
        "day": day, "ramp_active": ramp, "ramp_until": ramp_until,
        "used": used, "caps": caps, "remaining": remaining,
        "actions_last_hour": last_hour,
        "burst_cap": burst_hr, "burst_active": burst}, indent=2))
    sys.exit(0)

label = {"like": "Likes", "reply": "Replies", "follow": "Follows",
         "dm": "DMs (dm+followup)", "post": "Posts", "quote": "Quotes",
         "discover": "Discoveries"}
print(f"X action budget for {day}")
print(f"{'action':<20} {'used':>5} {'cap':>5} {'left':>5}")
print("-" * 38)
for k in ("like", "reply", "follow", "dm", "post", "quote", "discover"):
    flag = "  <-- CAP HIT" if remaining[k] == 0 else ""
    print(f"{label[k]:<20} {used[k]:>5} {caps[k]:>5} {remaining[k]:>5}{flag}")
print(f"\nactions in the last 60 min: {last_hour} (burst cap {burst_hr})")
if burst:
    print("  BURST WARNING: too many actions in one hour. Pause and spread out.")
if ramp:
    print(f"  COLD-START RAMP ACTIVE until {ramp_until}: caps are 25% of the normal limits.")
PY
