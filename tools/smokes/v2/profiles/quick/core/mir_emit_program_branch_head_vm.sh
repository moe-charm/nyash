#!/bin/bash
# mir_emit_program_branch_head_vm.sh — Runner emits MIR(JSON) for a program with a branch

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/mir_emit_branch_$$"
mkdir -p "$TMP_DIR"
PROG="$TMP_DIR/branch.hako"
JSON_OUT="$TMP_DIR/branch.json"

cat > "$PROG" << 'SRC'
static box Main { main(){
  local x = 0
  if x < 1 { x = 7 }
  return x
}}
SRC

set +e
"$NYASH_BIN" --emit-mir-json "$JSON_OUT" --backend mir "$PROG" >/dev/null 2>&1
RC=$?
set -e
if [ $RC -ne 0 ]; then
  test_fail "mir_emit_program_branch_head_vm" "runner failed rc=$RC"
  rm -rf "$TMP_DIR"; exit 1
fi

if grep -q '"functions"' "$JSON_OUT"; then
  test_pass "mir_emit_program_branch_head_vm"
  rm -rf "$TMP_DIR"; exit 0
else
  test_fail "mir_emit_program_branch_head_vm" "no functions in JSON"
  rm -rf "$TMP_DIR"; exit 1
fi
