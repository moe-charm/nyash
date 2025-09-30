#!/usr/bin/env bash
# call_trace_samples.sh — Run three representative call-trace parity checks
# Usage: tools/dev/call_trace_samples.sh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "Nyash binary not found: $BIN (run 'cargo build --release')" >&2
  exit 2
fi

SCRIPTS="$ROOT_DIR/tools/dev/call_trace_diff.sh"
if [[ ! -x "$SCRIPTS" ]]; then
  echo "call_trace_diff.sh not found: $SCRIPTS" >&2
  exit 2
fi

PASS=0; FAIL=0

run_case() {
  local name="$1"; shift
  echo "\n=== Case: $name ===" >&2
  if "$SCRIPTS" "$@"; then true; fi
  # Evaluate summary line
  local tmp="/tmp/ct_$$.log"
  "$SCRIPTS" "$@" >"$tmp" 2>&1 || true
  if grep -q "Result: OK (VM ⊆ LLVM)" "$tmp"; then
    echo "[OK] $name" >&2; PASS=$((PASS+1))
  else
    echo "[DIFF] $name" >&2; FAIL=$((FAIL+1))
  fi
  rm -f "$tmp"
}

# 1) json_lint (Method/Global)
run_case "json_lint" "$ROOT_DIR/apps/examples/json_lint/main.nyash" --kinds 'Method,Global'

# 2) array_min_ops (Method/Global)
run_case "array_min_ops" "$ROOT_DIR/apps/tests/array_min_ops.nyash" --kinds 'Method,Global'

# 3) selfhost compiler (emit json/mir)
run_case "selfhost_compiler_emit" "$ROOT_DIR/apps/selfhost-compiler/compiler.nyash" --args '-- --min-json --emit-mir' --kinds 'Global,Method'

echo "\nSummary: PASS=$PASS FAIL=$FAIL" >&2
[[ $FAIL -eq 0 ]]

