#!/usr/bin/env bash
set -u

# runner.sh — continuous self-healing agent runner for the x_bot passes.
#
# Starts all pass agents in detached tmux sessions, then watches them forever.
# If any session dies or the tmux server itself goes down, it restarts them.
#
# Usage:
#   bash runner.sh          run in foreground (starts + watches agents forever)
#   nohup bash runner.sh &  keep running after terminal closes
#   bash runner.sh --stop   stop everything (agents + this runner)
#
# Cadence (seconds), overridable via env:
#   MENTION_INTERVAL  (default 10)      Mention Bot Pass
#   DISCOVER_INTERVAL (default 60)      SaaS Lead Discovery Pass
#   CONTENT_INTERVAL  (default 21600)   Content & Strategy Pass

REPO_DIR="/home/adnan/x_bot"
LOOP_SCRIPT="$REPO_DIR/bin/run_pass_loop.sh"
PROMPTS_DIR="$REPO_DIR/bin/prompts"
WATCH_SECONDS="${WATCH_SECONDS:-30}"

MENTION_INTERVAL="${MENTION_INTERVAL:-10}"
DISCOVER_INTERVAL="${DISCOVER_INTERVAL:-180}"
CONTENT_INTERVAL="${CONTENT_INTERVAL:-7200}"

PASSES="pitch-server pass-tunnel pass-tunnelsync pass-mention pass-discover pass-content"
RUNNER_LOCK="$REPO_DIR/data/runner.pid"

# Headless virtual display for the Playwright browsers (X :0 needs root auth,
# so the passes use their own Xvfb display).
XVFB_DISPLAY="${XVFB_DISPLAY:-:99}"
if [ -z "${DISPLAY:-}" ]; then
  export DISPLAY="$XVFB_DISPLAY"
fi

ensure_xvfb() {
  if DISPLAY="$XVFB_DISPLAY" timeout 5 xset q >/dev/null 2>&1; then
    return 0
  fi
  if ! pgrep -f "Xvfb $XVFB_DISPLAY" >/dev/null 2>&1; then
    Xvfb "$XVFB_DISPLAY" -screen 0 1280x900x24 >/dev/null 2>&1 &
    sleep 2
    log "started Xvfb on $XVFB_DISPLAY"
  fi
}

mkdir -p "$REPO_DIR/data"

log() { echo "[runner $(date '+%F %T')] $*"; }

stop_all() {
  for name in $PASSES; do
    if tmux has-session -t "$name" 2>/dev/null; then
      tmux kill-session -t "$name" && log "stopped $name"
    fi
  done
  if [ -f "$RUNNER_LOCK" ]; then
    local pid
    pid="$(cat "$RUNNER_LOCK" 2>/dev/null)"
    if [ -n "${pid:-}" ] && [ "$pid" != "$$" ]; then
      kill "$pid" 2>/dev/null && log "stopped runner pid $pid"
    fi
    rm -f "$RUNNER_LOCK"
  fi
}

if [ "${1:-}" = "--stop" ]; then
  stop_all
  exit 0
fi

if [ -f "$RUNNER_LOCK" ]; then
  local_pid="$(cat "$RUNNER_LOCK" 2>/dev/null)"
  if kill -0 "${local_pid:-0}" 2>/dev/null; then
    log "runner already running (pid $local_pid). Not starting a second one."
    exit 0
  fi
fi
echo $$ > "$RUNNER_LOCK"
log "runner pid $$ started."

start_pass() {
  local name="$1" interval="$2" prompt_file="$3"
  if tmux has-session -t "$name" 2>/dev/null; then
    log "session '$name' already running"
    return
  fi
  log "starting '$name' (every ${interval}s)"
  tmux new-session -d -s "$name" -c "$REPO_DIR" \
    "bash $LOOP_SCRIPT $name $interval $PROMPTS_DIR/$prompt_file; exec bash"
}

start_server() {
  if ! tmux has-session -t pitch-server 2>/dev/null; then
    log "starting Rust webhook server 'pitch-server'..."
    tmux new-session -d -s pitch-server -c "$REPO_DIR" \
      "bash -c './target/release/pitch-cli server --port 8790; exec bash'"
  fi
}

start_tunnel() {
  if ! tmux has-session -t pass-tunnel 2>/dev/null; then
    log "starting localtunnel 'pass-tunnel' on port 8790..."
    tmux new-session -d -s pass-tunnel -c "$REPO_DIR" \
      "bash -c 'npx localtunnel --port 8790 --subdomain pitch-bot-adnan; exec bash'"
  fi
}

start_tunnelsync() {
  if ! tmux has-session -t pass-tunnelsync 2>/dev/null; then
    log "starting tunnel auto-sync daemon 'pass-tunnelsync'..."
    tmux new-session -d -s pass-tunnelsync -c "$REPO_DIR" \
      "bash -c 'python3 $REPO_DIR/bin/sync_tunnel_to_vercel.py; exec bash'"
  fi
}

while true; do
  # If tmux server is gone, all sessions died with it — restart everything.
  if ! tmux ls >/dev/null 2>&1; then
    log "tmux server down; restarting it"
    tmux start-server
    sleep 2
  fi

  ensure_xvfb
  start_server
  start_tunnel
  start_tunnelsync

  # Refresh real tech & developer memes from 4chan /g/ and Reddit periodically
  if [ ! -f "$REPO_DIR/data/fetched_memes.json" ] || [ $(($(date +%s) - $(stat -c %Y "$REPO_DIR/data/fetched_memes.json" 2>/dev/null || echo 0))) -gt 3600 ]; then
    python3 "$REPO_DIR/bin/fetch_memes.py" >/dev/null 2>&1 &
  fi

  # Refresh influencer radar & early infiltration targets every 3 minutes
  if [ ! -f "$REPO_DIR/data/influencer_radar.json" ] || [ $(($(date +%s) - $(stat -c %Y "$REPO_DIR/data/influencer_radar.json" 2>/dev/null || echo 0))) -gt 180 ]; then
    python3 "$REPO_DIR/bin/orbit_monitor.py" >/dev/null 2>&1 &
  fi

  start_pass pass-mention  "$MENTION_INTERVAL"  pass-mention.txt
  start_pass pass-discover "$DISCOVER_INTERVAL" pass-discover.txt
  start_pass pass-content  "$CONTENT_INTERVAL"  pass-content.txt

  sleep "$WATCH_SECONDS"
done
