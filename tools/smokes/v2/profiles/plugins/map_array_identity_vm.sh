#!/bin/bash
# map_array_identity_vm.sh — Plugins: Map stores Array handle identity check

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_map_array_identity_vm() {
  local code='static box Main { main() {
    local arr = new ArrayBox()
    arr.push(1)
    local m = new MapBox()
    m.set("list", arr)
    local arr2 = m.get("list")
    arr.push(2)
    if arr.size() == 2 && arr2.size() == 2 { print("share-ok") } else { print("share-ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local last; last=$(echo "$out" | tail -n 1)
  if [[ "$last" == "share-ok" ]]; then
    return 0
  else
    compare_outputs "share-ok" "$last" "map_array_identity_vm"
  fi
}

run_test "map_array_identity_vm" test_map_array_identity_vm
