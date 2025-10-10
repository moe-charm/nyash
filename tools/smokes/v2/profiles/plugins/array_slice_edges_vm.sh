#!/bin/bash
# array_slice_edges_vm.sh — Plugins: Array.slice edge cases

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_array_slice_edges_vm() {
  local code='static box Main { main() {
    local a = new ArrayBox(); a.push(1); a.push(2); a.push(3)
    local s1 = a.slice(0, 10)
    local s2 = a.slice(-5, 2)
    print(s1.size())
    print(s2.size())
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local last2; last2=$(echo "$out" | tail -n 2 | tr '\n' '|')
  # Expect s1.size=3 and s2.size=2 (if negative start clamps to 0)
  if [[ "$last2" == *"3|2|"* ]]; then
    return 0
  else
    compare_outputs "3|2|" "$last2" "array_slice_edges_vm"
  fi
}

run_test "array_slice_edges_vm" test_array_slice_edges_vm
