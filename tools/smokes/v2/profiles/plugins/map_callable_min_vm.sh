#!/bin/bash
# map_callable_min_vm.sh — MapBox stores CallableBox and calls it (sync/async)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_map_callable_min_vm() {
  export HAKO_CALLABLE_ASYNC=1
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
  local fr = m.callAsync("f", args)
  print("" + fr)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code" )
  # Expect sync=2 and async=Future string
  first=$(echo "$out" | grep -v "^Result:" | head -n 1 | tr -d '
')
  second=$(echo "$out" | grep -v "^Result:" | tail -n 1 | tr -d '
')
  if [ "$first" != "2" ]; then { test_fail "sync result not 2 (got "$first")"; return 1; }; fi
  echo "$second" | grep -E "Future|<future>" >/dev/null || { test_fail "async return not Future-like (got "$second")"; return 1; }
  test_pass "map_callable_min_vm"
}

run_test "map_callable_min_vm" test_map_callable_min_vm
