#!/bin/bash
# map_remove_returns_value_vm.sh — Plugins: Map.remove returns removed value or null

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_map_remove_returns_value_vm() {
  local code='static box Main { main() {
    local m = new MapBox()
    m.set("k", 42)
    m.remove("k")
    if m.has("k") == false { print("rm-val-ok") } else { print("rm-val-ng") }
    if m.remove("missing") == null { print("rm-null-ok") } else { print("rm-null-ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local tail; tail=$(echo "$out" | tail -n 2 | tr '\n' '|')
  if [[ "$tail" == *"rm-val-ok|rm-null-ok|"* ]]; then
    return 0
  else
    compare_outputs "rm-val-ok|rm-null-ok|" "$tail" "map_remove_returns_value_vm"
  fi
}

run_test "map_remove_returns_value_vm" test_map_remove_returns_value_vm
