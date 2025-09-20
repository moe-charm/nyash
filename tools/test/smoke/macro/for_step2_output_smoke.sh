#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/loopform/for_step2.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$("$bin" --backend vm "$src" 2>/dev/null || true)
# 0,2,4 が出力されることを簡易確認
echo "$out" | rg -q "^0$" && echo "$out" | rg -q "^2$" && echo "$out" | rg -q "^4$" && { echo "[OK] for_step2 output"; exit 0; }
echo "[FAIL] for_step2 output mismatch" >&2
echo "$out" >&2
exit 2

