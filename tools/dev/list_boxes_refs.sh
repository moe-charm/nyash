#!/usr/bin/env bash
# list_boxes_refs.sh — list remaining references to legacy src/boxes

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

PATTERN='\bcrate::boxes::|^\s*pub mod boxes;|use crate::boxes::'

usage() {
  cat <<USAGE
Usage: tools/dev/list_boxes_refs.sh [--by-dir] [--context N]

Options
  --by-dir     Show per-directory summary (top-level under src/)
  --context N  Show N lines of context for matches (default: 0)
USAGE
}

BY_DIR=0
CTX=0
while [ $# -gt 0 ]; do
  case "$1" in
    --by-dir) BY_DIR=1; shift ;;
    --context) CTX="${2:-0}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1"; usage; exit 2 ;;
  esac
done

echo "[boxes-refs] Searching for crate::boxes references..." >&2
if [ "${CTX}" != "0" ]; then
  rg -n "${PATTERN}" -S src -C "$CTX" || true
else
  rg -n "${PATTERN}" -S src || true
fi

echo "[boxes-refs] Summary:" >&2
FILES=$(find src -type f -name "*.rs" -print0 | xargs -0 rg -l "${PATTERN}" -S | wc -l)
LINES=$(rg -n "${PATTERN}" -S src | wc -l)
echo " files: ${FILES}"
echo " lines: ${LINES}"

if [ "$BY_DIR" = "1" ]; then
  echo "[boxes-refs] By directory (src/*):" >&2
  rg -n "${PATTERN}" -S src \
    | awk -F: '{print $1}' \
    | sed -E 's#^src/([^/]+).*$#\1#' \
    | sort | uniq -c | sort -nr
fi

echo "[boxes-refs] Tip: try plugin-only build for a quick check:" >&2
echo "  cargo build --release --no-default-features -F cli,plugins,host-anchors" >&2
