#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/if/assign_both_branches.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MIR_TRACE_HINTS=1

# Capture stderr; VM backend is sufficient (MIR builder runs)
out=$({ "$bin" --backend vm "$src" 1>/dev/null; } 2>&1 || true)

echo "$out" | rg -q "\[mir\]\[hint\] JoinResult\(x\)" || {
  echo "[FAIL] missing JoinResult(x) hint" >&2
  printf '%s\n' "$out" | tail -n 60 >&2
  exit 2
}

echo "[OK] MIR hints JoinResult trace smoke passed"

