#!/bin/bash
# selfhost_callable_call_nonarray_type_error_vm.sh — cb.call(nonArray) should fail (type/arity)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_selfhost_callable_call_nonarray_type_error_vm() {
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  local cb = a.methodRef("size", 0)
  // call with non-array (invalid)
  local r = cb.call(123)
  print("" + r)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  rc=$?
  if [ $rc -eq 0 ]; then
    echo "$out"
    test_fail "expected non-zero exit for cb.call(nonArray)"
    return 1
  fi
  test_pass "selfhost_callable_call_nonarray_type_error_vm"
}

run_test "selfhost_callable_call_nonarray_type_error_vm" test_selfhost_callable_call_nonarray_type_error_vm

