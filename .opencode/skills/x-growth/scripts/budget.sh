#!/usr/bin/env bash
# budget.sh — today's remaining X-action budget from state/activity-log.jsonl
#
# The agent runs this at the start of every session (and before any action type
# it's unsure about) to stay under the daily caps in safety.md.
#
# Usage:
#   bash .opencode/skills/x-growth/scripts/budget.sh            # today (local date)
#   bash .opencode/skills/x-growth/scripts/budget.sh 2026-06-08 # a specific day
#   bash .opencode/skills/x-growth/scripts/budget.sh --json     # machine-readable
#
# Only actions with "result":"ok" count toward caps. The DM cap covers both
# "dm" and "followup". Caps below mirror safety.md — keep them in sync. For a
# new/cold account, override with the env vars (e.g. CAP_DM=3) for the first
# few weeks.
set -euo pipefail

# --- caps (mirror safety.md) -------------------------------------------------
CAP_LIKE="${CAP_LIKE:-50}"
CAP_REPLY="${CAP_REPLY:-15}"
CAP_FOLLOW="${CAP_FOLLOW:-15}"
CAP_DM="${CAP_DM:-10}"          # dm + followup combined
CAP_POST="${CAP_POST:-4}"
CAP_QUOTE="${CAP_QUOTE:-4}"
CAP_DISCOVER="${CAP_DISCOVER:-40}"

# --- locate the log relative to this script ----------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"            # scripts/ -> skill root (contains state/)
LOG="$ROOT/state/activity-log.jsonl"

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

DAY="$DAY" JSON="$JSON" \
CAP_LIKE="$CAP_LIKE" CAP_REPLY="$CAP_REPLY" CAP_FOLLOW="$CAP_FOLLOW" \
CAP_DM="$CAP_DM" CAP_POST="$CAP_POST" CAP_QUOTE="$CAP_QUOTE" \
CAP_DISCOVER="$CAP_DISCOVER" \
python3 - "$LOG" <<'PY'
import json, os, sys

log = sys.argv[1]
day = os.environ["DAY"]
as_json = os.environ["JSON"] == "1"
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
except FileNotFoundError:
    pass

remaining = {k: max(0, caps[k] - used[k]) for k in caps}

if as_json:
    print(json.dumps({"day": day, "used": used, "caps": caps,
                      "remaining": remaining}, indent=2))
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
PY
