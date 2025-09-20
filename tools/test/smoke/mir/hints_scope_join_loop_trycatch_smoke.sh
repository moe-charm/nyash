#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_PARSER_STAGE3=1
export NYASH_MIR_HINTS="trace|scope|join"

src1="apps/tests/macro/exception/expr_postfix_direct.nyash"
out=$({ "$bin" --backend vm "$root/$src1" 1>/dev/null; } 2>&1 || true)

# Accept placeholder ids for now; assert presence only (exception scope)
echo "$out" | rg -F -q "[mir][hint] ScopeEnter(" || { echo "[FAIL] missing ScopeEnter in try/catch/cleanup case" >&2; echo "$out" >&2; exit 2; }

# Now check join on a basic if-assign case (existing sample)
src2="apps/tests/macro/if/assign_both_branches.nyash"
out2=$({ "$bin" --backend vm "$root/$src2" 1>/dev/null; } 2>&1 || true)
echo "$out2" | rg -F -q "[mir][hint] JoinResult(" || { echo "[FAIL] missing JoinResult in simple if-assign case" >&2; echo "$out2" >&2; exit 2; }

echo "[OK] MIR hints (scope+join) observed in loop+trycatch case"
exit 0
