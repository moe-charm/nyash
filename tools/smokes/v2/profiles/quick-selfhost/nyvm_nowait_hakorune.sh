#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
function test_body(){
  local app="$NYASH_ROOT/apps/tests/async-nowait-basic/main.hako"
  local out
  ensure_hako_toml
  out=$(NYASH_VM_MAX_INSTRUCTIONS=5000000 NYASH_DISABLE_PLUGINS=1 HAKO_NYVM_ENGINE=hakorune HAKO_ALLOW_USING_FILE=1 NYASH_USING_AST=1 "$NYASH_BIN" --backend nyvm "$app" 2>&1 | filter_noise | grep -v "^Unknown backend:" )
  
  compare_outputs """" "${out}" "nyvm-nowait-hakorune"
}
run_test "nyvm-nowait-hakorune" test_body || exit 1
print_summary
