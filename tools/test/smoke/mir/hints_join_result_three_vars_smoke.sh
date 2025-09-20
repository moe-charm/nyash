#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/if/assign_three_vars.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MIR_TRACE_HINTS=1
out=$({ "$bin" --backend vm "$src"; } 2>&1 || true)
echo "$out" | rg -q "\[mir\]\[hint\] JoinResult\(a\)" || { echo "[FAIL] missing JoinResult(a)" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q "\[mir\]\[hint\] JoinResult\(b\)" || { echo "[FAIL] missing JoinResult(b)" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q "\[mir\]\[hint\] JoinResult\(c\)" || { echo "[FAIL] missing JoinResult(c)" >&2; echo "$out" >&2; exit 2; }
echo "[OK] MIR hints JoinResult for three vars"
exit 0

