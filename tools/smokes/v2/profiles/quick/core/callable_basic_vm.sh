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
    if run_nyash_vm -c "$code" --dev >/dev/null; then
    test_pass "callable_basic_vm"
  else
    test_fail "callable_basic_vm" "non-zero rc"
    return 1
  fi

run_test "callable_basic_vm" test_callable_basic_vm
}
