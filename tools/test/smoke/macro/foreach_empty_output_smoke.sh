#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/loopform/foreach_empty.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$("$bin" --backend vm "$src" 2>/dev/null || true)
# 空配列なので出力なし（空行も不可）
if [ -z "${out//$'\n'/}" ]; then
  echo "[OK] foreach_empty output (no lines)"; exit 0
fi
echo "[FAIL] foreach_empty produced output unexpectedly" >&2
echo "$out" >&2
exit 2

