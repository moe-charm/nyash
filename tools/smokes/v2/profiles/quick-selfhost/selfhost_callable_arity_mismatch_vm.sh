#!/bin/bash
# selfhost_callable_arity_mismatch_vm.sh — Callable arity mismatch should Fail‑Fast

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_selfhost_callable_arity_mismatch_vm() {
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  local cb = a.methodRef("size", 0)
  // Intentionally pass one argument to size()/0
  local args = new ArrayBox()
  args.push(123)
  // This should Fail‑Fast (arity mismatch)
  local r = cb.call(args)
  print("" + r)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  rc=$?
  # Expect non-zero exit due to Fail‑Fast arity mismatch
  if [ $rc -eq 0 ]; then
    echo "$out"
    test_fail "selfhost_callable_arity_mismatch_vm expected non-zero exit"
    return 1
  fi
  test_pass "selfhost_callable_arity_mismatch_vm"
}

run_test "selfhost_callable_arity_mismatch_vm" test_selfhost_callable_arity_mismatch_vm

