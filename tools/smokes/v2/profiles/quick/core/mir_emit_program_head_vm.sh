#!/bin/bash
# mir_emit_program_head_vm.sh — Runner emits MIR(JSON) header for a minimal program

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/mir_emit_head_$$"
mkdir -p "$TMP_DIR"
PROG="$TMP_DIR/min.hako"
JSON_OUT="$TMP_DIR/min.json"

cat > "$PROG" << 'SRC'
// minimal program: return 7
static box Main { main(){ return 7 } }
SRC

set +e
"$NYASH_BIN" --emit-mir-json "$JSON_OUT" --backend mir "$PROG" >/dev/null 2>&1
RC=$?
set -e
if [ $RC -ne 0 ]; then
  test_fail "mir_emit_program_head_vm" "runner failed rc=$RC"
  rm -rf "$TMP_DIR"; exit 1
fi

if grep -q '"kind":"Program"' "$JSON_OUT" || grep -q '"functions"' "$JSON_OUT"; then
  test_pass "mir_emit_program_head_vm"
  rm -rf "$TMP_DIR"; exit 0
else
  test_fail "mir_emit_program_head_vm" "no Program header in JSON"
  rm -rf "$TMP_DIR"; exit 1
fi
