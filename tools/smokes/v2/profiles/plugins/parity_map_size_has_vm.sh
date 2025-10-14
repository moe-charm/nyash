#!/bin/bash
# parity_map_size_has_vm.sh — Plugins suite: minimal MapBox size/has parity

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_parity_map_size_has_vm() {
  local code='static box Main { main() {
    local m = new MapBox()
    m.set("k","v")
    print(m.size())
    print(m.has("k"))
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" | filter_noise)
  # Expect two lines: 1 and true
  local last2
  last2=$(echo "$out" | grep -E '^(true|false|[0-9]+)$' | tail -n 2 | tr '\n' ',')
  if [[ "$last2" == *"1,true"* ]]; then
    test_pass parity_map_size_has_vm
  else
    compare_outputs "1,true" "$last2" "parity_map_size_has_vm"
  fi
}

run_test "parity_map_size_has_vm" test_parity_map_size_has_vm

