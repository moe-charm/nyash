#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/exception/loop_postfix_sugar.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_CATCH_NEW=1
out=$("$bin" --backend vm "$src" 2>/dev/null || true)
exp=$'cleanup\ncleanup'
if [ "$(printf '%s' "$out" | tr -d '\r')" != "$(printf '%s' "$exp")" ]; then
  echo "[FAIL] loop_postfix_sugar produced unexpected output" >&2
  echo "--- got ---" >&2; printf '%s\n' "$out" >&2
  echo "--- exp ---" >&2; printf '%s\n' "$exp" >&2
  exit 2
fi

echo "[OK] loop_postfix_catch_cleanup output matched"
exit 0

