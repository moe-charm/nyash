#!/bin/bash
# set_add_has_size_vm.sh — Plugins suite: minimal Set ops via nyrt.set.* externs

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_set_add_has_size_vm() {
  local code='static box Main { main() {
    local s = new SetBox()
    s.add(1)
    s.add(1)
    print(s.size())
    print(s.has(1))
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" 2>&1 | filter_noise)
  local last2
  last2=$(echo "$out" | grep -E '^(true|false|[0-9]+)$' | tail -n 2 | tr '\n' ',')
  if [[ "$last2" == *"1,true"* ]]; then
    test_pass set_add_has_size_vm
  else
    compare_outputs "1,true" "$last2" "set_add_has_size_vm"
  fi
}

run_test "set_add_has_size_vm" test_set_add_has_size_vm
