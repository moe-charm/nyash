#!/usr/bin/env bash
# mircall_module_function_vm.sh — sanity: ModuleFunction call via MIR (ArrayBox.size)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_module_function_vm() {
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push(7)\n    if a.size() != 1 { return 201 }\n    return 0\n  }\n}\n'
  out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_module_function_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_module_function_vm test_mircall_module_function_vm
