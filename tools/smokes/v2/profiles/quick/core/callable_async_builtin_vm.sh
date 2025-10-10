#!/bin/bash
# callable_async_builtin_vm.sh — Builtin receiver callAsync spawns via scheduler and resolves via Future

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_callable_async_builtin_vm() {
  export HAKO_CALLABLE_ASYNC=1
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  // Callable from instance method size/0
  local cb = a.methodRef("size", 0)
  local args = new ArrayBox()
  // async call on builtin receiver
  local fut = cb.callAsync(args)
  // Give the VM some instructions to poll the scheduler
  local i = 0
  loop(i < 1000) { i = i + 1 }
  // Print the Future — it should be ready with numeric result
  print("" + fut)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code" --dev)
  echo "$out" | tr -d '\r' | grep -E '^(Future\(ready: 2\)|<future>)$' >/dev/null || { test_fail "missing async builtin result=2"; return 1; }
  test_pass "callable_async_builtin_vm"
}

run_test "callable_async_builtin_vm" test_callable_async_builtin_vm
