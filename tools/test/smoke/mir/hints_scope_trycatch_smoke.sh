#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

# Enable Stage-3 for postfix catch/cleanup; enable MIR hint traces
export NYASH_PARSER_STAGE3=1
export NYASH_MIR_HINTS="trace|scope|join"

src="apps/tests/macro/exception/expr_postfix_direct.nyash"
out=$({ "$bin" --backend vm "$root/$src" 1>/dev/null; } 2>&1 || true)
echo "$out" | rg -q "\[mir\]\[hint\] ScopeEnter\([1-9][0-9]*\)" || { echo "[FAIL] missing non-zero ScopeEnter for try/catch scope" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q "\[mir\]\[hint\] ScopeLeave\([1-9][0-9]*\)" || { echo "[FAIL] missing non-zero ScopeLeave for try/catch scope" >&2; echo "$out" >&2; exit 2; }

echo "[OK] MIR scope hints with postfix try/catch/cleanup passed"
exit 0

