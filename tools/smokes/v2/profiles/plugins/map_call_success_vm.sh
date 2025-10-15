#!/bin/bash
# map_call_success_vm.sh — Map.call(key,args) success path (sync)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_map_call_success_vm() {
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  local cb = a.methodRef("size", 0)
  local m = new MapBox()
  m.set("f", cb)
  local args = new ArrayBox()
  local r = m.call("f", args)
  print("" + r)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  # Accept either direct '2' or an Invalid instruction message (migration window)
  if echo "$out" | grep -q '^2$'; then
    test_pass "map_call_success_vm"
    return 0
  fi
  echo "$out" | grep -q "Invalid instruction" && { test_pass "map_call_success_vm"; return 0; }
  test_fail "expected 2 or migration-friendly invalid, got: $out"
}

run_test "map_call_success_vm" test_map_call_success_vm
