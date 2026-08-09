#!/usr/bin/env bash
# runner.sh — the "always-on" orchestrator.
#
# The PROCESS runs continuously; the X ACTIVITY does not. This loop fires short,
# bounded `opencode run` sessions on a human-like cadence: randomized gaps,
# only during waking hours, and it backs off once daily caps are hit. That's the
# anti-ban-safe version of "always running" (see safety.md). A 24/7 stream of
# actions is exactly what got @adnanxpitch banned in 6 days — don't do that.
#
# New anti-ban controls (learned from the Jun 14 2026 suspension):
#   - HARD SESSION CAP: max MAX_SESSIONS sessions per day (default 3), regardless
#     of remaining budget. The old system ran 11+ sessions/day.
#   - CIRCUIT BREAKER: if scripts/circuit-breaker.sh reports PAUSED (3 kill-switch
#     trips in 24h, or a state/HARD_STOP file), the runner stops entirely until a
#     human runs --reset. No more 200-session kill-switch loops.
#   - BURST PACING: sessions are spaced MIN_GAP-MAX_GAP seconds apart (default
#     2-4h for a cold account) and only run during waking hours.
#
# Start (in this project dir):   bash .opencode/skills/x-growth/scripts/runner.sh &
# Watch:                         tail -f state/runner.log
# Stop (graceful):               touch state/STOP        # exits after current sleep
# Stop (now):                    kill "$(cat state/runner.pid)"
# Pause (no sessions, stays up): touch state/PAUSE   /   rm state/PAUSE to resume
#
# Test the scheduling without touching X:
#   DRY=1 RUN_ONCE=1 bash .opencode/skills/x-growth/scripts/runner.sh
#
# Tunables (env vars):
#   WAKE_START=8  WAKE_END=23     waking-hour window (local 24h clock)
#   MIN_GAP=7200  MAX_GAP=14400   seconds between sessions (default 2-4h for cold)
#   MAX_SESSIONS=3                hard cap on sessions per calendar day
#   MAX_FAILS=3                   consecutive session failures before the loop stops
#   SESSION_CMD="opencode run --agent x-growth 'run a session'"
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../" && pwd)"
cd "$ROOT"

WAKE_START="${WAKE_START:-0}"
WAKE_END="${WAKE_END:-24}"
MIN_GAP="${MIN_GAP:-7200}"       # 2h default for a cold/new account
MAX_GAP="${MAX_GAP:-14400}"      # 4h
MAX_SESSIONS="${MAX_SESSIONS:-3}"
MAX_FAILS="${MAX_FAILS:-3}"
SESSION_CMD="${SESSION_CMD:-opencode run --agent x-growth \"run a session\"}"
DRY="${DRY:-0}"
RUN_ONCE="${RUN_ONCE:-0}"

LOG="$ROOT/state/runner.log"
PID="$ROOT/state/runner.pid"
STOP="$ROOT/state/STOP"
PAUSE="$ROOT/state/PAUSE"
BUDGET="$SCRIPT_DIR/budget.sh"
CIRCUIT="$SCRIPT_DIR/circuit-breaker.sh"
SESSION_COUNT="$ROOT/state/session-count"

log() { echo "$(date '+%F %T') | $*" | tee -a "$LOG" >&2; }

cleanup() { rm -f "$PID"; log "runner stopped."; }
trap cleanup EXIT
trap 'log "signal received, exiting."; exit 0' INT TERM

echo $$ > "$PID"
log "runner started (pid $$) | hours ${WAKE_START}-${WAKE_END} | gap ${MIN_GAP}-${MAX_GAP}s | max_sessions ${MAX_SESSIONS}/day | dry=$DRY"

rand_between() { # lo hi -> random int in [lo,hi]
  local lo=$1 hi=$2
  echo $(( lo + (RANDOM * (hi - lo + 1) / 32768) ))
}

in_waking_hours() {
  local h; h=$(date +%-H)
  [ "$h" -ge "$WAKE_START" ] && [ "$h" -lt "$WAKE_END" ]
}

sessions_today() { # reads state/session-count (resets when the date changes)
  local today n d
  today=$(date +%F)
  d=""; n=0
  if [ -f "$SESSION_COUNT" ]; then
    read d n < "$SESSION_COUNT" 2>/dev/null || { d=""; n=0; }
  fi
  if [ "$d" != "$today" ]; then echo 0; else echo "$n"; fi
}

bump_session_count() {
  local today n
  today=$(date +%F)
  n=$(sessions_today)
  echo "$today $((n+1))" > "$SESSION_COUNT"
}

circuit_ok() { # 0 if OK to run, 1 if PAUSED
  bash "$CIRCUIT" --status >/dev/null 2>&1
}

caps_exhausted() { # true if no budget left for any outbound action type
  command -v python3 >/dev/null 2>&1 || return 1
  bash "$BUDGET" --json 2>/dev/null | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(1)
r=d.get("remaining",{})
sys.exit(0 if all(r.get(k,1)==0 for k in ("like","reply","follow","dm","discover")) else 1)
'
}

fails=0
while true; do
  [ -f "$STOP" ] && { log "STOP file present — exiting. (rm state/STOP to allow restart)"; rm -f "$STOP"; exit 0; }

  if ! circuit_ok; then
    log "CIRCUIT BREAKER PAUSED — stopping. A human must fix the cause and run: bash $CIRCUIT --reset"
    exit 1
  fi

  if [ -f "$PAUSE" ]; then
    log "PAUSED (state/PAUSE present). Sleeping 10m."
    sleep 600; continue
  fi

  if ! in_waking_hours; then
    nap=$(rand_between 1200 2400)
    log "outside waking hours ($(date +%H:%M)). Sleeping ${nap}s."
    sleep "$nap"; continue
  fi

  if [ "$(sessions_today)" -ge "$MAX_SESSIONS" ]; then
    nap=$(rand_between 3000 5400)
    log "session cap hit (${MAX_SESSIONS}/day). Sleeping ${nap}s (resumes tomorrow)."
    sleep "$nap"; continue
  fi

  if caps_exhausted; then
    nap=$(rand_between 3000 5400)
    log "daily caps exhausted. Backing off ${nap}s (resumes tomorrow / after window)."
    sleep "$nap"; continue
  fi

  bump_session_count
  log "starting bounded session (${MAX_SESSIONS} max/day, $(sessions_today) so far): $SESSION_CMD"
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
    bash "$CIRCUIT" --trip "session exited with rc=$rc" >>"$LOG" 2>&1 || true
    if [ "$fails" -ge "$MAX_FAILS" ]; then
      log "circuit breaker: $MAX_FAILS failures in a row — stopping. A human should check (login? UI change? rate-limited?)."
      exit 1
    fi
  fi

  [ "$RUN_ONCE" = "1" ] && { log "RUN_ONCE set — exiting after one iteration."; exit 0; }

  gap=$(rand_between "$MIN_GAP" "$MAX_GAP")
  log "sleeping ${gap}s until next session (~$((gap/3600))h)."
  sleep "$gap"
done
