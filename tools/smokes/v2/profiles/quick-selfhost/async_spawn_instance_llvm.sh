#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
require_llvm_or_skip || { print_summary; exit 0; }

function test_body(){
  local app="$NYASH_ROOT/apps/tests/async-spawn-instance/main.hako"
  local out
  ensure_hako_toml
  out=$(PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" NYASH_LLVM_USE_HARNESS=1 HAKO_BACKEND=llvm "$NYASH_BIN" "$app" 2>&1 | grep -v "^📊 MIR Module compiled successfully" \
    | grep -v "^📊 Functions:" \
    | grep -v "Nyash LLVM Backend - Executing file:" \
    | grep -v "SMOKES bypass" \
    | filter_noise)
  compare_outputs """" "${out}" "async-spawn-instance-llvm"
}
run_test "async-spawn-instance LLVM" test_body || exit 1
print_summary