#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/loopform/two_vars.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MIR_TRACE_HINTS=1

out=$({ "$bin" --backend vm "$src" 1>/dev/null; } 2>&1 || true)

# Check the LoopCarrier hint contains both variable names (order-agnostic)
echo "$out" | rg -q "\[mir\]\[hint\] LoopCarrier\((i,sum|sum,i)\)" || {
  echo "[FAIL] missing LoopCarrier(i,sum) hint" >&2
  printf '%s\n' "$out" | tail -n 80 >&2
  exit 2
}

echo "[OK] MIR hints LoopCarrier(two vars) trace smoke passed"

