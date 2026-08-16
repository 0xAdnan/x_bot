#!/usr/bin/env bash
set -e

REPO_DIR="/home/adnan/x_bot"
CARGO_ENV="$HOME/.cargo/env"

echo "=== STARTING PITCH AGENTS IN ISOLATED TMUX SESSIONS ==="

# 1. Start Webhook Server session
if tmux has-session -t pitch-server 2>/dev/null; then
    echo "[Tmux] Session 'pitch-server' is already running."
else
    echo "[Tmux] Starting session 'pitch-server'..."
    tmux new-session -d -s pitch-server -c "$REPO_DIR" \
      "bash -c 'source $CARGO_ENV && ./target/release/pitch-cli server --port 8790; exec bash'"
fi

# 2. Start 10s Trigger & Queue Worker session
if tmux has-session -t pitch-trigger 2>/dev/null; then
    echo "[Tmux] Session 'pitch-trigger' is already running."
else
    echo "[Tmux] Starting session 'pitch-trigger'..."
    tmux new-session -d -s pitch-trigger -c "$REPO_DIR" \
      "bash -c 'python3 /home/adnan/x_bot/bin/rust_trigger_loop.py; exec bash'"
fi

# 3. Start Discovery Agent session
if tmux has-session -t pitch-discover 2>/dev/null; then
    echo "[Tmux] Session 'pitch-discover' is already running."
else
    echo "[Tmux] Starting session 'pitch-discover'..."
    tmux new-session -d -s pitch-discover -c "$REPO_DIR" \
      "bash -c 'source $CARGO_ENV && ./target/release/pitch-cli discover --max 5; exec bash'"
fi

# 4. Start YC & Antler Startup Scout session
if tmux has-session -t pitch-yc-antler 2>/dev/null; then
    echo "[Tmux] Session 'pitch-yc-antler' is already running."
else
    echo "[Tmux] Starting session 'pitch-yc-antler'..."
    tmux new-session -d -s pitch-yc-antler -c "$REPO_DIR" \
      "bash -c 'python3 /home/adnan/x_bot/bin/rust_yc_antler_loop.py; exec bash'"
fi

echo ""
echo "=== TMUX AGENTS STARTED SUCCESSFULLY ==="
echo "You can monitor any agent's live progress at any time using:"
echo "  • Webhook Server:       tmux attach -t pitch-server"
echo "  • Trigger Worker:       tmux attach -t pitch-trigger"
echo "  • Discovery Agent:      tmux attach -t pitch-discover"
echo "  • YC & Antler Scout:    tmux attach -t pitch-yc-antler"
echo ""
echo "To detach from a tmux session without stopping the agent, press: Ctrl+B then D"
