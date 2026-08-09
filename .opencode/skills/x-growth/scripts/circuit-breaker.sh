#!/usr/bin/env bash
# circuit-breaker.sh — auto-pause after repeated kill-switch/failed sessions.
#
# Why this exists: the old account got 200+ consecutive "kill-switch" sessions
# that kept re-running instead of stopping. That is exactly what burned the
# account. This file makes the pause mechanical, not something the agent has to
# remember to respect.
#
# Usage:
#   bash .opencode/skills/x-growth/scripts/circuit-breaker.sh --status
#       Prints status. Exit 0 = OK to run. Exit 1 = PAUSED (do not run).
#   bash .opencode/skills/x-growth/scripts/circuit-breaker.sh --trip "reason"
#       Agent calls this on ANY kill-switch / CAPTCHA / warning / 3x failure.
#       After MAX_CONSECUTIVE trips in 24h it creates state/HARD_STOP.
#   bash .opencode/skills/x-growth/scripts/circuit-breaker.sh --reset
#       HUMAN action: clears the pause (after fixing the root cause).
#
# Only trips within the last 24h count as "consecutive". A day with no trips
# decays the count, but an existing state/HARD_STOP still blocks until reset.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../" && pwd)"
STATE="$ROOT/state"
LOG="$STATE/circuit-breaker.jsonl"
STOPFILE="$STATE/HARD_STOP"
MAX_CONSECUTIVE="${MAX_CONSECUTIVE:-3}"
WINDOW="${WINDOW:-86400}"   # 24h

command -v python3 >/dev/null 2>&1 || { echo "circuit-breaker.sh needs python3." >&2; exit 1; }

count_trips() { # prints count of trips within the window
  python3 - "$LOG" "$WINDOW" <<'PY'
import json, sys, time
log, win = sys.argv[1], int(sys.argv[2])
now = time.time()
n = 0
try:
    with open(log) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                ts = float(json.loads(line).get("ts", 0))
            except (json.JSONDecodeError, TypeError, ValueError):
                continue
            if now - ts <= win:
                n += 1
except FileNotFoundError:
    pass
print(n)
PY
}

status() {
  local n; n=$(count_trips)
  echo "circuit-breaker: $n trip(s) in the last 24h (limit $MAX_CONSECUTIVE)."
  if [ -f "$STOPFILE" ]; then
    echo "STATUS: PAUSED (state/HARD_STOP present)."
    echo "  Reason: $(cat "$STOPFILE")"
    echo "  Fix the root cause, then a HUMAN must run: bash $0 --reset"
    exit 1
  fi
  if [ "$n" -ge "$MAX_CONSECUTIVE" ]; then
    echo "STATUS: PAUSED ($n consecutive trips)."
    echo "  Fix the root cause, then a HUMAN must run: bash $0 --reset"
    exit 1
  fi
  echo "STATUS: OK to run."
  exit 0
}

trip() {
  local reason="${1:-no reason given}"
  local ts; ts=$(date +%s)
  printf '{"ts":%s,"reason":"%s"}\n' "$ts" "$reason" >> "$LOG"
  local n; n=$(count_trips)
  echo "trip recorded ($n/$MAX_CONSECUTIVE in 24h): $reason"
  if [ "$n" -ge "$MAX_CONSECUTIVE" ]; then
    printf 'Circuit breaker tripped at %s after %s consecutive trips in 24h. Last reason: %s\n' \
      "$(date '+%F %T')" "$n" "$reason" > "$STOPFILE"
    echo "PAUSED. Created $STOPFILE. Automation must stop until a human fixes the cause and runs --reset."
  fi
}

reset() {
  rm -f "$STOPFILE"
  : > "$LOG"
  echo "Circuit breaker reset. Automation may resume."
}

case "${1:-}" in
  --status) status ;;
  --trip)   trip "${2:-}" ;;
  --reset)  reset ;;
  *)
    echo "usage: $0 --status | --trip \"reason\" | --reset" >&2
    exit 2 ;;
esac
