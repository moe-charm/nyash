#!/bin/bash
# array_slice_edges_vm.sh — Plugins: Array.slice edge cases

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_array_slice_edges_vm() {
  export SMOKES_CLEAN_ENV=0
  export SMOKES_DEFAULT_TIMEOUT=0
  local code='static box Main { main() {
    local a = new ArrayBox()
    a.push(1)
    a.push(2)
    a.push(3)
    local s1 = a.slice(0, 10)
    local s2 = a.slice(-5, 2)
    if s1.size() == 3 { print("slice1-ok") } else { print("slice1-ng") }
    if s2.size() == 2 { print("slice2-ok") } else { print("slice2-ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local last2; last2=$(echo "$out" | tail -n 2 | tr '\n' '|')
  # Expect slice lengths (stage2 plugin handle to plugin array)
  if [[ "$last2" == *"slice1-ok|slice2-ok|"* ]]; then
    return 0
  else
    compare_outputs "slice1-ok|slice2-ok|" "$last2" "array_slice_edges_vm"
  fi
}

run_test "array_slice_edges_vm" test_array_slice_edges_vm
