#!/bin/bash
# selfhost_callable_async_parallel_vm.sh — callAsync twice and ensure both complete/present

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_selfhost_callable_async_parallel_vm() {
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  local cb = a.methodRef("size", 0)
  local args = new ArrayBox()
  local f1 = cb.callAsync(args)
  local f2 = cb.callAsync(args)
  // spin a bit to let scheduler advance
  local i = 0
  loop(i < 8000) { i = i + 1 }
  print("" + f1)
  print("" + f2)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  # Accept either pending or ready form for both futures
  ready=$(echo "$out" | tr -d '\r' | grep -c '^Future(ready: 2)$' || true)
  pending=$(echo "$out" | tr -d '\r' | grep -c '^<future>$' || true)
  total=$((ready + pending))
  if [ "$total" -lt 2 ]; then
    echo "$out"
    test_fail "expected two Future lines (ready or pending). got $total"
    return 1
  fi
  test_pass "selfhost_callable_async_parallel_vm"
}

run_test "selfhost_callable_async_parallel_vm" test_selfhost_callable_async_parallel_vm

