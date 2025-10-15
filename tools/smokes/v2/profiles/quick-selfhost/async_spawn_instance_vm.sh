#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
function test_body(){
  local app="$NYASH_ROOT/apps/tests/async-spawn-instance/main.hako"
  local out
  ensure_hako_toml
  out=$(HAKO_BACKEND=vm "$NYASH_BIN" "$app" 2>&1 | filter_noise)
  compare_outputs """" "${out}" "async-spawn-instance-vm"
}
run_test "async-spawn-instance VM" test_body || exit 1
print_summary
