#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/if/assign_two_vars.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MIR_TRACE_HINTS=1
out=$({ "$bin" --backend vm "$src"; } 2>&1 || true)
echo "$out" | rg -q "\[mir\]\[hint\] JoinResult\(x\)" || { echo "[FAIL] missing JoinResult(x)" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q "\[mir\]\[hint\] JoinResult\(y\)" || { echo "[FAIL] missing JoinResult(y)" >&2; echo "$out" >&2; exit 2; }
echo "[OK] MIR hints JoinResult for two vars"
exit 0

