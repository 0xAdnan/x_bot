#!/usr/bin/env bash
set -e

# start_pass_agents.sh — launches the 3 continuous OpenCode agent passes in
# detached tmux sessions so they keep running after the terminal is closed.
# Default cadence (seconds) can be overridden via env vars:
#   MENTION_INTERVAL  (default 10)     Mention Bot Pass
#   DISCOVER_INTERVAL (default 60)     SaaS Lead Discovery Pass (full-time)
#   CONTENT_INTERVAL  (default 21600)  Content & Strategy Pass (4x/day)

REPO_DIR="/home/adnan/x_bot"
PROMPTS_DIR="$REPO_DIR/bin/prompts"
LOOP_SCRIPT="$REPO_DIR/bin/run_pass_loop.sh"
OPENCODE_BIN="${OPENCODE_BIN:-$(command -v opencode || echo "$HOME/.opencode/bin/opencode")}"

MENTION_INTERVAL="${MENTION_INTERVAL:-10}"
DISCOVER_INTERVAL="${DISCOVER_INTERVAL:-60}"
CONTENT_INTERVAL="${CONTENT_INTERVAL:-21600}"

start_pass() {
  local name="$1" interval="$2" prompt_file="$3"
  if tmux has-session -t "$name" 2>/dev/null; then
    echo "[tmux] session '$name' already running, skipping."
    return 0
  fi
  echo "[tmux] starting '$name' (every ${interval}s)..."
  tmux new-session -d -s "$name" -c "$REPO_DIR" \
    "bash $LOOP_SCRIPT $name $interval $PROMPTS_DIR/$prompt_file; exec bash"
}

echo "=== STARTING CONTINUOUS OPENCODE AGENT PASSES ==="
echo "opencode: $OPENCODE_BIN"

start_pass pass-mention  "$MENTION_INTERVAL"  pass-mention.txt
start_pass pass-discover "$DISCOVER_INTERVAL" pass-discover.txt
start_pass pass-content  "$CONTENT_INTERVAL"  pass-content.txt

echo ""
echo "=== ALL PASSES RUNNING IN DETACHED TMUX SESSIONS ==="
echo "  tmux attach -t pass-mention    (Mention Bot)"
echo "  tmux attach -t pass-discover   (SaaS Lead Discovery)"
echo "  tmux attach -t pass-content    (Content & Strategy)"
echo "Detach with Ctrl+B then D. Logs: data/logs/pass-*.log"
