#!/usr/bin/env bash
set -e

REPO_DIR="/home/adnan/x_bot"
CARGO_ENV="$HOME/.cargo/env"

if tmux has-session -t pitch-dashboard 2>/dev/null; then
    echo "[Tmux] Session 'pitch-dashboard' already running."
else
    echo "[Tmux] Creating tiled multi-pane dashboard 'pitch-dashboard'..."
    
    # Pane 1: Server
    tmux new-session -d -s pitch-dashboard -c "$REPO_DIR" \
      "bash -c 'source $CARGO_ENV && ./target/release/pitch-cli server --port 8790; exec bash'"
    
    # Split horizontally for Pane 2: Trigger Loop
    tmux split-window -h -t pitch-dashboard:0 -c "$REPO_DIR" \
      "bash -c 'python3 /home/adnan/x_bot/bin/rust_trigger_loop.py; exec bash'"
    
    # Split vertically for Pane 3: Discovery Agent
    tmux split-window -v -t pitch-dashboard:0.0 -c "$REPO_DIR" \
      "bash -c 'source $CARGO_ENV && ./target/release/pitch-cli discover --max 5; exec bash'"
    
    tmux select-layout -t pitch-dashboard tiled
fi

echo "To view all 3 agents simultaneously in split-screen layout:"
echo "  tmux attach -t pitch-dashboard"
