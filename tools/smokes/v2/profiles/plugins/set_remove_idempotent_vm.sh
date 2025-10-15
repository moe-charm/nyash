#!/bin/bash
# set_remove_idempotent_vm.sh — Plugins suite: Set remove idempotency via nyrt.set.*

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_set_remove_idempotent_vm() {
  local code='static box Main { main() {
    local s = new SetBox()
    s.add(2)
    s.remove(2)
    s.remove(2)
    print(s.has(2))
    return 0
  }}'
  local out
  out_full=$(run_nyash_vm -c "$code" 2>&1 | filter_noise)
  out=$(echo "$out_full" | tail -n 1)
  compare_outputs "false" "$out" "set_remove_idempotent_vm"
}

run_test "set_remove_idempotent_vm" test_set_remove_idempotent_vm
