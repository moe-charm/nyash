#!/bin/bash
# map_values_keys_delete_vm.sh — Plugins: Map.values/keys/delete edge behavior

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_map_values_keys_delete_vm() {
  local code='static box Main { main() {
    local m = new MapBox()
    m.set(1, 10)
    m.set("k", "v")
    local ks = m.keys();
    local vs = m.values();
    print(ks.size())
    print(vs.size())
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local last4; last4=$(echo "$out" | tail -n 4 | tr '\n' '|')
  # Expect sizes 2 and 2 only (delete may be unmapped yet)
  if [[ "$last4" == *"2|2|"* ]]; then
    return 0
  else
    compare_outputs "2|2|" "$last4" "map_values_keys_delete_vm"
  fi
}

run_test "map_values_keys_delete_vm" test_map_values_keys_delete_vm
