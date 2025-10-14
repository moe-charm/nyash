#!/bin/bash
# parity_array_size_vm.sh — Plugins suite: minimal ArrayBox size parity

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_parity_array_size_vm() {
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push(1)\n    a.push(2)\n    print(a.size())\n    return 0\n  }\n}'
  local out
  out=$(run_nyash_vm -c "$code" | filter_noise)
  if echo "$out" | grep -qx '2'; then
    test_pass parity_array_size_vm
  else
    compare_outputs "2" "$out" "parity_array_size_vm"
  fi
}

run_test "parity_array_size_vm" test_parity_array_size_vm

