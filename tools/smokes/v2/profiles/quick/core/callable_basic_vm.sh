#!/bin/bash
# callable_basic_vm.sh — Create CallableBox from instance method and invoke

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_callable_basic_vm() {
  local code='

static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  // make callable from instance method size/0
  local cb = a.methodRef("size", 0)
  // call with [] (no args) packed in an array for now
  local args = new ArrayBox()
  print(cb.arity())
  local r = cb.call(args)
  print("" + r)
  // (async path tested separately; keep smoke minimal)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code" --dev)
  # Expect two identical lines with size 2
  echo "$out" | tr -d '' | grep -E '^2$' >/dev/null || { test_fail "missing sync result=2"; return 1; }
    test_pass "callable_basic_vm"
}

run_test "callable_basic_vm" test_callable_basic_vm
