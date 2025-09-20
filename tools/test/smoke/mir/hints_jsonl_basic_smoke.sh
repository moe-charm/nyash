#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out_jsonl="$root/tmp/mir_hints_basic.jsonl"
rm -f "$out_jsonl"

# Emit scope + join hints into JSONL file (no stderr noise)
export NYASH_PARSER_STAGE3=1
export NYASH_MIR_HINTS="jsonl=$out_jsonl|scope|join"

# Run two small samples that exercise both kinds (errors are tolerated, we only need hints emission)
"$bin" --backend vm "$root/apps/tests/macro/exception/expr_postfix_direct.nyash" >/dev/null 2>&1 || true
"$bin" --backend vm "$root/apps/tests/macro/if/assign_both_branches.nyash" >/dev/null 2>&1 || true

test -s "$out_jsonl" || { echo "[FAIL] hints jsonl not created" >&2; exit 2; }

# Basic presence checks (don’t overfit exact ids)
rg -q '"kind":"ScopeEnter"' "$out_jsonl" || { echo "[FAIL] ScopeEnter not found in jsonl" >&2; exit 2; }
rg -q '"kind":"JoinResult"' "$out_jsonl" || { echo "[FAIL] JoinResult not found in jsonl" >&2; exit 2; }

echo "[OK] MIR hints JSONL basic smoke passed ($out_jsonl)"
exit 0

