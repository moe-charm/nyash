#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <tmux_session> \"Task A\" \"Task B\" [\"Task C\" ...]" >&2
  exit 1
fi

SESSION="$1"; shift
TASKS=("$@")
if [ ${#TASKS[@]} -lt 2 ]; then
  echo "Provide at least two task strings." >&2
  exit 1
fi

export CODEX_MAX_CONCURRENT=${CODEX_MAX_CONCURRENT:-2}
export CODEX_DEDUP=${CODEX_DEDUP:-1}
export CODEX_NOTIFY_MINIMAL=${CODEX_NOTIFY_MINIMAL:-1}

WORK_DIR="$HOME/.codex-async-work"
RUN_DIR="$WORK_DIR/running"
mkdir -p "$RUN_DIR"

idx=0
echo "[keep-two-loop] Maintaining ${CODEX_MAX_CONCURRENT} concurrent tasks. Ctrl-C to stop." >&2
while true; do
  # Count running by sentinel first, fallback by pgid
  RUN=0
  if [ -d "$RUN_DIR" ]; then
    RUN=$(ls -1 "$RUN_DIR"/codex-*.run 2>/dev/null | wc -l | tr -d ' ' || echo 0)
  fi
  if [ "${RUN:-0}" -eq 0 ] && command -v pgrep >/dev/null 2>&1; then
    RUN=$(pgrep -f -- 'codex .* exec' | xargs -r -I {} sh -c 'ps -o pgid= -p "$1" 2>/dev/null' _ {} | awk '{print $1}' | sort -u | wc -l | tr -d ' ' || echo 0)
  fi

  NEED=$((CODEX_MAX_CONCURRENT - ${RUN:-0}))
  if [ $NEED -gt 0 ]; then
    echo "[keep-two-loop] running=$RUN; starting $NEED task(s)…" >&2
    for ((i=0; i<NEED; i++)); do
      task="${TASKS[$idx]}"; idx=$(((idx+1) % ${#TASKS[@]}))
      CODEX_ASYNC_DETACH=1 ./tools/codex-async-notify.sh "$task" "$SESSION" >/dev/null 2>&1 || true
      sleep 0.2
    done
  fi
  sleep 2
done


