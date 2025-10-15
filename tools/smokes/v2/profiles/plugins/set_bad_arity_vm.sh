#!/bin/bash
# set_bad_arity_vm.sh — Plugins suite: Set bad arity/type should error (Fail-Fast)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_set_bad_arity_vm() {
  local code='static box Main { main() {
    local s = new SetBox()
    // bad: add() requires 1 arg
    s.add()
    return 0
  }}'
  out_full=$(run_nyash_vm -c "$code" 2>&1 | filter_noise)
  # Expect a normalized invalid_inst message
  last=$(echo "$out_full" | tail -n 1)
  case "$last" in
    SMOKES_ERR:*|Invalid\ instruction:*|Invalid\ value:*) test_pass set_bad_arity_vm ;;
    *) compare_outputs "SMOKES_ERR:" "$last" "set_bad_arity_vm" ;;
  esac
}

run_test "set_bad_arity_vm" test_set_bad_arity_vm
