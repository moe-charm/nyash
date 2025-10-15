#!/bin/bash
# map_call_missing_vm.sh — Map.call on missing key should return null

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_map_call_missing_vm() {
  local code='
static box Main { main() {
  local m = new MapBox()
  local args = new ArrayBox()
  local r = m.call("nope", args)
  print("" + r)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  # Spec-fixed: missing → null
  if echo "$out" | grep -qx 'null'; then
    test_pass "map_call_missing_vm"; return 0
  fi
  compare_outputs 'null' "$out" "map_call_missing_vm"
  return 1
}

run_test "map_call_missing_vm" test_map_call_missing_vm
