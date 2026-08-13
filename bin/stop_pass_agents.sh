#!/usr/bin/env bash
set -e

for name in pass-mention pass-discover pass-content; do
  if tmux has-session -t "$name" 2>/dev/null; then
    tmux kill-session -t "$name"
    echo "[tmux] stopped '$name'"
  else
    echo "[tmux] '$name' not running"
  fi
done
