#!/usr/bin/env bash
set -e

REPO_DIR="/home/adnan/x_bot"
CARGO_ENV="$HOME/.cargo/env"

if tmux has-session -t pitch-dashboard 2>/dev/null; then
    echo "[Tmux] Session 'pitch-dashboard' already running."
else
    echo "[Tmux] Creating multi-pane tiled dashboard 'pitch-dashboard'..."
    
    # Pane 1: Server
    tmux new-session -d -s pitch-dashboard -c "$REPO_DIR" \
      "bash -c 'source $CARGO_ENV && ./target/release/pitch-cli server --port 8790; exec bash'"
    
    # Pane 2: Trigger Loop (Mention + MCP Worker)
    tmux split-window -h -t pitch-dashboard:0 -c "$REPO_DIR" \
      "bash -c 'python3 /home/adnan/x_bot/bin/rust_trigger_loop.py; exec bash'"
    
    # Pane 3: Discovery Agent
    tmux split-window -v -t pitch-dashboard:0.0 -c "$REPO_DIR" \
      "bash -c 'source $CARGO_ENV && ./target/release/pitch-cli discover --max 5; exec bash'"
    
    # Pane 4: 24/7 AI Research Daemon
    tmux split-window -v -t pitch-dashboard:0.1 -c "$REPO_DIR" \
      "bash -c 'python3 /home/adnan/x_bot/bin/rust_researcher_loop.py; exec bash'"

    # Pane 5: YC & Antler Startup Scout
    tmux split-window -v -t pitch-dashboard:0.2 -c "$REPO_DIR" \
      "bash -c 'python3 /home/adnan/x_bot/bin/rust_yc_antler_loop.py; exec bash'"
    
    tmux select-layout -t pitch-dashboard tiled
fi

echo "To view all agents simultaneously in split-screen tiled dashboard:"
echo "  tmux attach -t pitch-dashboard"
