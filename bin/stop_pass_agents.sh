#!/usr/bin/env bash
set -e

for name in pass-mention pass-discover pass-yc-antler pass-content pitch-server pitch-trigger pitch-discover pitch-yc-antler pitch-dashboard; do
  if tmux has-session -t "$name" 2>/dev/null; then
    tmux kill-session -t "$name"
    echo "[tmux] stopped '$name'"
  else
    echo "[tmux] '$name' not running"
  fi
done
