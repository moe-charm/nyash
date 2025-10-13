#!/bin/bash
# map_values_array_element_vm.sh — values() returns Array where element can be an ArrayBox (HostHandle)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_map_values_array_element_vm() {
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(7)
  local m = new MapBox()
  m.set("x", a)
  local vals = m.values()
  print("VSZ:" + vals.size())
  local e0 = vals.get(0)
  // ensure we can call .size() on the element (ArrayBox)
  print("E0SZ:" + e0.size())
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  vsz=$(echo "$out" | awk -F: '/^VSZ:/{print $2; exit}')
  e0=$(echo "$out" | awk -F: '/^E0SZ:/{print $2; exit}')
  if [ "$vsz" = "1" ] && [ "$e0" = "1" ]; then
    test_pass "map_values_array_element_vm"
  else
    echo "$out"; test_fail "expected VSZ:1 and E0SZ:1"; return 1
  fi
}

run_test "map_values_array_element_vm" test_map_values_array_element_vm
