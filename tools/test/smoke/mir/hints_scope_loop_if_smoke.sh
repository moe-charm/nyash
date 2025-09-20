#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MIR_TRACE_HINTS=1

# Case 1: loop body induces scope enter/leave and loop header/latch hints
src1="apps/tests/macro/loopform/simple.nyash"
out1=$({ "$bin" --backend vm "$src1" 1>/dev/null; } 2>&1 || true)
echo "$out1" | rg -q "\[mir\]\[hint\] LoopHeader" || { echo "[FAIL] missing LoopHeader" >&2; exit 2; }
echo "$out1" | rg -q "\[mir\]\[hint\] LoopLatch" || { echo "[FAIL] missing LoopLatch" >&2; exit 2; }
echo "$out1" | rg -q "\[mir\]\[hint\] ScopeEnter\([1-9][0-9]*\)" || { echo "[FAIL] missing non-zero ScopeEnter for loop body" >&2; echo "$out1" >&2; exit 2; }
echo "$out1" | rg -q "\[mir\]\[hint\] ScopeLeave\([1-9][0-9]*\)" || { echo "[FAIL] missing non-zero ScopeLeave for loop body" >&2; echo "$out1" >&2; exit 2; }

# Case 2: if branches induce scope enter/leave
src2="apps/tests/macro/if/assign_two_vars.nyash"
out2=$({ "$bin" --backend vm "$src2" 1>/dev/null; } 2>&1 || true)
echo "$out2" | rg -q "\[mir\]\[hint\] ScopeEnter\([1-9][0-9]*\)" || { echo "[FAIL] missing non-zero ScopeEnter for if-branch" >&2; echo "$out2" >&2; exit 2; }
echo "$out2" | rg -q "\[mir\]\[hint\] ScopeLeave\([1-9][0-9]*\)" || { echo "[FAIL] missing non-zero ScopeLeave for if-branch" >&2; echo "$out2" >&2; exit 2; }

echo "[OK] MIR scope hints for loop and if passed"
exit 0

