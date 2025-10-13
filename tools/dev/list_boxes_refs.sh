#!/usr/bin/env bash
# list_boxes_refs.sh — list remaining references to legacy src/boxes

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

echo "[boxes-refs] Searching for crate::boxes references..." >&2
rg -n "\bcrate::boxes::|^\s*pub mod boxes;|use crate::boxes::" -S src || true

echo "[boxes-refs] Summary:" >&2
echo -n " files: "; find src -type f -name "*.rs" -print0 | xargs -0 rg -l "\bcrate::boxes::|^\s*pub mod boxes;|use crate::boxes::" -S | wc -l
echo -n " lines: "; rg -n "\bcrate::boxes::|^\s*pub mod boxes;|use crate::boxes::" -S src | wc -l

echo "[boxes-refs] Tip: try plugin-only build for a quick check:" >&2
echo "  cargo build --release --no-default-features -F cli,plugins,host-anchors" >&2

