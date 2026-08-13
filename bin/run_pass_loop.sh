#!/usr/bin/env bash
set -u

# run_pass_loop.sh — runs a single OpenCode x-growth agent pass in an infinite loop.
# Usage: run_pass_loop.sh <name> <interval_seconds> <prompt_file>
# env: OPENCODE_BIN (path to opencode binary, default: PATH lookup then ~/.opencode/bin/opencode)
#      PASS_MODEL (model id, default: google/gemini-3.6-flash)

REPO_DIR="/home/adnan/x_bot"
NAME="$1"
INTERVAL="${2:-300}"
PROMPT_FILE="$3"
PASS_MODEL="${PASS_MODEL:-google/gemini-3.6-flash}"
LOG_DIR="$REPO_DIR/data/logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/${NAME}.log"

OPENCODE_BIN="${OPENCODE_BIN:-$(command -v opencode)}"
if [ -z "$OPENCODE_BIN" ]; then
  OPENCODE_BIN="$HOME/.opencode/bin/opencode"
fi
if [ ! -x "$OPENCODE_BIN" ]; then
  echo "[pass:$NAME] ERROR: opencode binary not found at $OPENCODE_BIN" >&2
  exit 1
fi

if [ "$NAME" = "pass-discover" ]; then
  CONFIG_DIR="$REPO_DIR/config/adnanspitch"
else
  CONFIG_DIR="$REPO_DIR/config/trypitchdotco"
fi

echo "[pass:$NAME] starting loop every ${INTERVAL}s. config=$CONFIG_DIR log=$LOG_FILE"

while true; do
  echo "=== [$NAME] pass start $(date -u +%FT%TZ) ==="
  cd "$CONFIG_DIR" || exit 1
  "$OPENCODE_BIN" run --dir "$REPO_DIR" --agent x-growth -m "$PASS_MODEL" --dangerously-skip-permissions "$(cat "$PROMPT_FILE")" 2>&1 | tee -a "$LOG_FILE"
  echo "=== [$NAME] pass end $(date -u +%FT%TZ) — sleeping ${INTERVAL}s ==="
  sleep "$INTERVAL"
done
