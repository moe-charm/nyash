#!/bin/bash
# selfhost_callable_async_vm.sh — Hakorune VM path: methodRef → callAsync (Future)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_selfhost_callable_async_vm() {
  # async enabled by default in test_runner (HAKO_CALLABLE_ASYNC=1)
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  local cb = a.methodRef("size", 0)
  local args = new ArrayBox()
  local fut = cb.callAsync(args)
  // Poll the scheduler with a small busy loop to allow completion
  local i = 0
  loop(i < 5000) { i = i + 1 }
  print("" + fut)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  echo "$out" | tr -d '\r' | grep -E '^(Future\(ready: 2\)|<future>)$' >/dev/null || { echo "$out"; test_fail "selfhost_callable_async_vm expected Future(ready: 2) or <future>"; return 1; }
  test_pass "selfhost_callable_async_vm"
}

run_test "selfhost_callable_async_vm" test_selfhost_callable_async_vm
