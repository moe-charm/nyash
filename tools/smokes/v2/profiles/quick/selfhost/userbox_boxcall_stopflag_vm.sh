#!/bin/bash
# userbox_boxcall_stopflag_vm.sh — Disable user instance BoxCall and ensure method rewrite works

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2

test_userbox_boxcall_stopflag_vm() {
  local code=$'static box Main {\n  main() {\n    local f = new Foo()\n    print("" + f.inc(41))\n    return 0\n  }\n}\n\nbox Foo {\n  inc(x) { return x + 1 }\n}\n'
  # Stop fast-path: disallow user instance BoxCall; enforce prod profile rewrite
  NYASH_VM_USER_INSTANCE_BOXCALL=0 NYASH_USING_PROFILE=prod out=$(run_nyash_vm -c "$code" --dev | tail -n 1 | tr -d '\r')
  compare_outputs "42" "$out" "userbox_boxcall_stopflag_vm" || return 1
  return 0
}

run_test "userbox_boxcall_stopflag_vm" test_userbox_boxcall_stopflag_vm

