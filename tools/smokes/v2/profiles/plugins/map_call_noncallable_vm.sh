#!/bin/bash
# map_call_noncallable_vm.sh — Map.call on non-callable value should error

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_map_call_noncallable_vm() {
  local code='
static box Main { main() {
  local m = new MapBox()
  m.set("x", 123)
  local args = new ArrayBox()
  local r = m.call("x", args)
  print("" + r)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  rc=$?
  if [ $rc -eq 0 ]; then
    echo "$out"
    test_fail "expected non-zero exit for Map.call on non-callable"
    return 1
  fi
  test_pass "map_call_noncallable_vm"
}

run_test "map_call_noncallable_vm" test_map_call_noncallable_vm
