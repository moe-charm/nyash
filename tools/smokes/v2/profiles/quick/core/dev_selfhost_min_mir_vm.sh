#!/bin/bash
# dev_selfhost_min_mir_vm.sh — Selfhost compiler emits minimal MIR(JSON v0) via ENV透過（devゲート）

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate: enable explicitly to avoid noise in quick
if [ "${SMOKES_ENABLE_SELFHOST_ACCEPT:-0}" != "1" ]; then
  test_skip "dev_selfhost_min_mir_vm" "enable with SMOKES_ENABLE_SELFHOST_ACCEPT=1"
  exit 0
fi

# Use Selfhost runner path (parent→child ENV 透過):
# - NYASH_USE_NY_COMPILER=1 enables selfhost path
# - NYASH_NY_COMPILER_MIN_JSON=1 passes --min-json
# - NYASH_NY_COMPILER_CHILD_ARGS passes --emit-mir to child compiler

OUT=$(NYASH_USE_NY_COMPILER=1 \
      NYASH_NY_COMPILER_MIN_JSON=1 \
      NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir" \
      NYASH_JSON_ONLY=1 \
      timeout 5 "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/examples/string_p0.nyash" 2>/dev/null | \
      awk 'match($0,/^\{/) {print; exit}')

if echo "$OUT" | grep -q '"functions"' && echo "$OUT" | grep -q '"blocks"'; then
  test_pass "dev_selfhost_min_mir_vm"
  exit 0
else
  test_fail "dev_selfhost_min_mir_vm" "no MIR JSON head"
  exit 1
fi

