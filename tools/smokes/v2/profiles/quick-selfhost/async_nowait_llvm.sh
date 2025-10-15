#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

function test_body(){
  local app="$NYASH_ROOT/apps/tests/async-nowait-basic/main.hako"
  local out
  ensure_hako_toml
  echo "$NYASH_BIN" --backend llvm "$app"
  out=$(env PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" NYASH_LLVM_USE_HARNESS=1 "$NYASH_BIN" --backend llvm "$app" 2>&1 | grep -v '^📊 MIR Module compiled successfully' | grep -v '^📊 Functions:' | filter_noise)
  compare_outputs """" "${out}" "async-nowait-basic-llvm"
}
require_llvm_or_skip || { print_summary; exit 0; }

run_test "async-nowait-basic LLVM" test_body || exit 1
print_summary
