#!/usr/bin/env bash
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/.." && pwd)

SMOKE_FILE="${1:-$ROOT/tmp/smoke_print.nyash}"

if [[ ! -f "$SMOKE_FILE" ]]; then
  echo "error: smoke file not found: $SMOKE_FILE" >&2
  exit 2
fi

NYASH="${NYASH_BIN:-$ROOT/target/release/nyash}"
if [[ ! -x "$NYASH" ]]; then
  echo "error: nyash binary not found or not executable: $NYASH" >&2
  echo "hint: cargo build --release --features cranelift-jit" >&2
  exit 3
fi

echo "[VM] $SMOKE_FILE"
timeout 10s "$NYASH" --backend vm "$SMOKE_FILE" | sed -n '1,80p'

echo ""
echo "[JIT] $SMOKE_FILE"
timeout 12s "$NYASH" --backend jit "$SMOKE_FILE" | sed -n '1,120p'

echo ""
echo "✅ smoke done"

