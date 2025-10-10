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
    local ks = m.keys()
    local vs = m.values()
    if ks.size() == 2 { print("keys2") } else { print("keysNG") }
    if vs.size() == 2 { print("values2") } else { print("valuesNG") }
    m.remove(1)
    print("rm-called")
    if m.has("k") == true { print("has-k-ok") } else { print("has-k-ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local expect=("keys2" "values2" "rm-called" "has-k-ok")
  for token in "${expect[@]}"; do
    if [[ "$out" != *"$token"* ]]; then
      compare_outputs "${expect[*]}" "$out" "map_values_keys_delete_vm"
      return 1
    fi
  done
  return 0
}

run_test "map_values_keys_delete_vm" test_map_values_keys_delete_vm
