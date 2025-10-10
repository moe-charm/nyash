#!/bin/bash
# selfhost_callable_basic_vm.sh — Hakorune VM path: methodRef → call (sync)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_selfhost_callable_basic_vm() {
  local code='
static box Main { main() {
  local a = new ArrayBox()
  a.push(1)
  a.push(2)
  // Build callable from instance method size/0
  local cb = a.methodRef("size", 0)
  // call with [] (no args)
  local args = new ArrayBox()
  local r = cb.call(args)
  print("" + r)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  got=$(printf "%s\n" "$out" | tr -d '\r' | awk '/^[0-9]+$/{print; exit}')
  [ "$got" = "2" ] || { echo "$out"; test_fail "selfhost_callable_basic_vm expected 2 (got '$got')"; return 1; }
  test_pass "selfhost_callable_basic_vm"
}

run_test "selfhost_callable_basic_vm" test_selfhost_callable_basic_vm
