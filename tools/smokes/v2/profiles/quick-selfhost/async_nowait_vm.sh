#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

function test_body(){
  local app="$NYASH_ROOT/apps/tests/async-nowait-basic/main.hako"
  local out
  ensure_hako_toml
  echo "$NYASH_BIN" --backend vm "$app"
  out=$("$NYASH_BIN" --backend vm "$app" 2>&1 | filter_noise)
  compare_outputs """" "${out}" "async-nowait-basic-vm"
}

run_test "async-nowait-basic VM" test_body || exit 1
print_summary
