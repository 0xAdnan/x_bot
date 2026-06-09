#!/usr/bin/env bash
# runner.sh — the "always-on" orchestrator.
#
# The PROCESS runs continuously; the X ACTIVITY does not. This loop fires short,
# bounded `opencode run` sessions on a human-like cadence: randomized gaps,
# only during waking hours, and it backs off once daily caps are hit. That's the
# anti-ban-safe version of "always running" (see safety.md). A 24/7 stream of
# actions is exactly what gets accounts banned — don't do that.
#
# Start (in this project dir):   bash .opencode/skills/x-growth/scripts/runner.sh &
# Watch:                         tail -f state/runner.log
# Stop (graceful):               touch state/STOP        # exits after current sleep
# Stop (now):                    kill "$(cat state/runner.pid)"
# Pause (no sessions, stays up):  touch state/PAUSE   /   rm state/PAUSE to resume
#
# Test the scheduling without touching X:
#   DRY=1 RUN_ONCE=1 bash .opencode/skills/x-growth/scripts/runner.sh
#
# Tunables (env vars):
#   WAKE_START=8  WAKE_END=23     waking-hour window (local 24h clock)
#   MIN_GAP=1500  MAX_GAP=5400    seconds between sessions (default 25–90 min)
#   SESSION_CMD="opencode run --agent x-growth 'run a session'"
#   MAX_FAILS=3                   consecutive session failures before the loop stops
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
cd "$ROOT"

WAKE_START="${WAKE_START:-5}"
WAKE_END="${WAKE_END:-23}"
MIN_GAP="${MIN_GAP:-1500}"
MAX_GAP="${MAX_GAP:-5400}"
MAX_FAILS="${MAX_FAILS:-3}"
SESSION_CMD="${SESSION_CMD:-opencode run --agent x-growth \"run a session\"}"
DRY="${DRY:-0}"
RUN_ONCE="${RUN_ONCE:-0}"

LOG="$ROOT/state/runner.log"
PID="$ROOT/state/runner.pid"
STOP="$ROOT/state/STOP"
PAUSE="$ROOT/state/PAUSE"
BUDGET="$SCRIPT_DIR/budget.sh"

log() { echo "$(date '+%F %T') | $*" | tee -a "$LOG" >&2; }

cleanup() { rm -f "$PID"; log "runner stopped."; }
trap cleanup EXIT
trap 'log "signal received, exiting."; exit 0' INT TERM

echo $$ > "$PID"
log "runner started (pid $$) | hours ${WAKE_START}-${WAKE_END} | gap ${MIN_GAP}-${MAX_GAP}s | dry=$DRY"

rand_between() { # lo hi -> random int in [lo,hi]
  local lo=$1 hi=$2
  echo $(( lo + (RANDOM * (hi - lo + 1) / 32768) ))
}

in_waking_hours() {
  local h; h=$(date +%-H)
  [ "$h" -ge "$WAKE_START" ] && [ "$h" -lt "$WAKE_END" ]
}

caps_exhausted() { # true if no budget left for any outbound action type
  command -v python3 >/dev/null 2>&1 || return 1
  bash "$BUDGET" --json 2>/dev/null | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(1)
r=d.get("remaining",{})
# exhausted only when every action type is at 0
sys.exit(0 if all(r.get(k,1)==0 for k in ("like","reply","follow","dm","discover")) else 1)
'
}

fails=0
while true; do
  [ -f "$STOP" ] && { log "STOP file present — exiting. (rm state/STOP to allow restart)"; rm -f "$STOP"; exit 0; }

  if [ -f "$PAUSE" ]; then
    log "PAUSED (state/PAUSE present). Sleeping 10m."
    sleep 600; continue
  fi

  if ! in_waking_hours; then
    nap=$(rand_between 1200 2400)
    log "outside waking hours ($(date +%H:%M)). Sleeping ${nap}s."
    sleep "$nap"; continue
  fi

  if caps_exhausted; then
    nap=$(rand_between 3000 5400)
    log "daily caps exhausted. Backing off ${nap}s (resumes tomorrow / after window)."
    sleep "$nap"; continue
  fi

  log "starting bounded session: $SESSION_CMD"
  if [ "$DRY" = "1" ]; then
    log "[dry-run] would run the session now."
    rc=0
  else
    if eval "$SESSION_CMD" >>"$LOG" 2>&1; then rc=0; else rc=$?; fi
  fi

  if [ "$rc" -eq 0 ]; then
    fails=0
    log "session finished ok."
  else
    fails=$((fails+1))
    log "session FAILED (rc=$rc). consecutive failures: $fails/$MAX_FAILS."
    if [ "$fails" -ge "$MAX_FAILS" ]; then
      log "circuit breaker: $MAX_FAILS failures in a row — stopping. A human should check (login? UI change? rate-limited?)."
      exit 1
    fi
  fi

  [ "$RUN_ONCE" = "1" ] && { log "RUN_ONCE set — exiting after one iteration."; exit 0; }

  gap=$(rand_between "$MIN_GAP" "$MAX_GAP")
  log "sleeping ${gap}s until next session (~$((gap/60)) min)."
  sleep "$gap"
done
