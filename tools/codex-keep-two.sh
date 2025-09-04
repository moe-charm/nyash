#!/bin/bash
# Keep two codex exec jobs running by topping up when fewer are active.
# Usage:
#   tools/codex-keep-two.sh <tmux_session> "Task A" "Task B" ["Task C" ...]
# Notes:
#   - Detects only real `codex exec` processes via ps/awk (avoids self-matches).
#   - Starts tasks in order, cycling if more top-ups are needed.

set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <tmux_session> \"Task A\" \"Task B\" [\"Task C\" ...]" >&2
  exit 1
fi

SESSION="$1"; shift
TASKS=("$@")
if [ ${#TASKS[@]} -eq 0 ]; then
  echo "Provide at least one task string." >&2
  exit 1
fi

CODEX_PROC_PATTERN=${CODEX_PROC_PATTERN:-'codex .* exec'}

list_running() {
  if command -v pgrep >/dev/null 2>&1; then
    pgrep -af -- "$CODEX_PROC_PATTERN" | grep -v 'pgrep' || true
  else
    ps -eo pid=,args= | grep -E -- "$CODEX_PROC_PATTERN" | grep -v grep || true
  fi
}

count_running() {
  list_running | wc -l | tr -d ' '\
  || echo 0
}

RUNNING=$(count_running)
echo "[keep-two] 実際のcodexプロセス数: ${RUNNING}"
NEED=$((2 - RUNNING))
if [ $NEED -le 0 ]; then
  echo "[keep-two] already running: $RUNNING (>=2)."
  exit 0
fi

echo "[keep-two] running=$RUNNING, starting $NEED top-up task(s)."

idx=0
for ((i=0; i<NEED; i++)); do
  TASK="${TASKS[$idx]}"; idx=$(((idx+1) % ${#TASKS[@]}))
  echo "[keep-two] start: $TASK"
  CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "$TASK" "$SESSION" >/dev/null 2>&1 || true
done

echo "[keep-two] now running: $(count_running)"
