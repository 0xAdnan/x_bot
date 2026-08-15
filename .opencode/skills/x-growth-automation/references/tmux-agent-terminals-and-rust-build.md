# Tmux Multi-Agent Terminals & Rust Build Workflow

This reference governs the configuration of detached tmux agent sessions, multi-pane split-screen terminal dashboards, and the local Rust toolchain compilation workflow for `pitch-cli`.

## 1. Isolated Tmux Agent Sessions
To allow operators to inspect agent progress in real time while keeping background processes immune to terminal window closure:

- Each agent or daemon runs in its own named `tmux` session.
- Append `; exec bash` to the execution command so that if a single-pass command completes, the terminal window remains open for log inspection.

### Recommended Tmux Agent Sessions
```bash
# 1. Webhook Server Session
tmux new-session -d -s pitch-server -c "/home/adnan/x_bot" \
  "bash -c 'source $HOME/.cargo/env && ./target/release/pitch-cli server --port 8790; exec bash'"

# 2. 10-Second Mention & MCP Trigger Loop Session
tmux new-session -d -s pitch-trigger -c "/home/adnan/x_bot" \
  "bash -c 'python3 /home/adnan/x_bot/bin/rust_trigger_loop.py; exec bash'"

# 3. SaaS Prospect Discovery Session
tmux new-session -d -s pitch-discover -c "/home/adnan/x_bot" \
  "bash -c 'source $HOME/.cargo/env && ./target/release/pitch-cli discover --max 5; exec bash'"
```

## 2. Split-Screen Multi-Pane Tiled Dashboard (`pitch-dashboard`)
For unified monitoring of all 3 agents simultaneously in a single terminal view:

```bash
# Create multi-pane tiled tmux dashboard
tmux new-session -d -s pitch-dashboard -c "/home/adnan/x_bot" \
  "bash -c 'source $HOME/.cargo/env && ./target/release/pitch-cli server --port 8790; exec bash'"

tmux split-window -h -t pitch-dashboard:0 -c "/home/adnan/x_bot" \
  "bash -c 'python3 /home/adnan/x_bot/bin/rust_trigger_loop.py; exec bash'"

tmux split-window -v -t pitch-dashboard:0.0 -c "/home/adnan/x_bot" \
  "bash -c 'source $HOME/.cargo/env && ./target/release/pitch-cli discover --max 5; exec bash'"

tmux select-layout -t pitch-dashboard tiled
```

### Attachment & Inspection Commands
- **Attach to Tiled Dashboard:** `tmux attach -t pitch-dashboard`
- **Attach to Standalone Session:** `tmux attach -t pitch-server` / `pitch-trigger` / `pitch-discover`
- **Detach from Tmux:** Press `Ctrl + B` then `D`.
- **Capture Live Terminal Pane:** `tmux capture-pane -t pitch-trigger -p`

## 3. Rust Toolchain Compilation & Config Override (`dotenvy::from_path_override`)
- **Cargo Release Build:** Compile the `pitch-cli` binary using `. "$HOME/.cargo/env" && cargo build --release`.
- **Config Override:** In `src/config.rs`, always use `dotenvy::from_path_override(&env_path)` rather than `dotenvy::from_path(&env_path)` so updated `.env` values correctly override stale environment variables.
- **Local Verification First:** Run ad-hoc verification scripts (`/tmp/hermes-verify-*.py`) to confirm binary execution and Axum health on port `8790`.
- **Local Git Policy:** Commit changes locally. Do NOT push to GitHub unless explicitly requested by the user.
