#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/match/literal_basic.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MIR_TRACE_HINTS=1

# Capture stderr (where hints are printed)
out=$({ "$bin" --backend vm "$src" 1>/dev/null; } 2>&1 || true)

echo "$out" | rg -q "\[mir\]\[hint\] ScopeEnter\(0\)" || {
  echo "[FAIL] missing ScopeEnter(0) hint" >&2
  exit 2
}
echo "$out" | rg -q "\[mir\]\[hint\] ScopeLeave\(0\)" || {
  echo "[FAIL] missing ScopeLeave(0) hint" >&2
  exit 2
}

echo "[OK] MIR hints trace smoke passed"

