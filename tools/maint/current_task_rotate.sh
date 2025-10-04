#!/usr/bin/env bash
set -euo pipefail

# Rotate CURRENT_TASK.md when it grows too large.
# - Archives full content to docs/development/current_task_archive/CURRENT_TASK_YYYYMMDD-HHMMSS.md
# - Creates a fresh CURRENT_TASK.md skeleton and appends the last N lines as Recent Log
#
# Env or args:
#   --threshold-kb <N>      default 64
#   --keep-lines  <N>       default 400
#   --archive-dir <path>    default docs/development/current_task_archive
#
THRESH_KB=64
KEEP_LINES=400
ARCHIVE_DIR="docs/development/current_task_archive"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --threshold-kb) THRESH_KB=${2:-64}; shift 2;;
    --keep-lines)   KEEP_LINES=${2:-400}; shift 2;;
    --archive-dir)  ARCHIVE_DIR=${2:-docs/development/current_task_archive}; shift 2;;
    *) echo "[rotate] unknown arg: $1" >&2; exit 2;;
  esac
done

FILE="CURRENT_TASK.md"
if [[ ! -f "$FILE" ]]; then
  echo "[rotate] $FILE not found; nothing to do" >&2
  exit 0
fi

SIZE_BYTES=$(wc -c < "$FILE" | tr -d ' ')
LIMIT_BYTES=$(( THRESH_KB * 1024 ))
if (( SIZE_BYTES <= LIMIT_BYTES )); then
  echo "[rotate] size ${SIZE_BYTES}B <= ${LIMIT_BYTES}B; skip" >&2
  exit 0
fi

mkdir -p "$ARCHIVE_DIR"
STAMP=$(date +%Y%m%d-%H%M%S)
ARCHIVE_FILE="$ARCHIVE_DIR/CURRENT_TASK_${STAMP}.md"
cp -f "$FILE" "$ARCHIVE_FILE"

echo "[rotate] archived to $ARCHIVE_FILE" >&2

# Build new skeleton
cat > "$FILE" << 'EOF'
# CURRENT_TASK — Now & Next

## Today
- 

## Next
- 

## Risks / Blockers
- 

## Notes
- 

## Recent Log (carryover)
EOF

# Append last N lines from archive into Recent Log
# (ensure we don't exceed the same threshold again immediately)
tail -n "$KEEP_LINES" "$ARCHIVE_FILE" >> "$FILE" || true

echo "[rotate] wrote new skeleton with last ${KEEP_LINES} lines as carryover" >&2
