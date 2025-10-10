#!/bin/bash
# callable_async_plugin_vm.sh — Plugin receiver callAsync spawns and resolves via Future

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_callable_async_plugin_vm() {
  # Force async path
  export HAKO_CALLABLE_ASYNC=1
  local code='
static box Main { main() {
  // Use MapBox receiver to ensure plugin-backed path in plugins profile
  local a = new ArrayBox()
  // build callable from instance method size/0
  local cb = a.methodRef("size", 0)
  local args = new ArrayBox()
  a.push(1)
  a.push(2)
  // async
  local fut = cb.callAsync(args)
  print("" + fut)
  // no wait
  return 0
}}
'
  out=$(run_nyash_vm -c "$code" --dev)
  echo "$out" | tr -d '
' | grep -E 'Future|<future>' >/dev/null || { test_fail "async return not Future-like"; return 1; }
  test_pass "callable_async_plugin_vm"
}

run_test "callable_async_plugin_vm" test_callable_async_plugin_vm
