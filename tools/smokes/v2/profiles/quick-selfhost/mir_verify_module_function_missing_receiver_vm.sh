#!/usr/bin/env bash
# mir_verify_module_function_missing_receiver_vm.sh
# Ensure Verifier catches ModuleFunction static calls missing the singleton 'me'.

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_verify_module_function_missing_receiver_vm() {
  local code=$'static box Foo {\n  hello(a) { return a; }\n}\nstatic box Main {\n  main() {\n    call("Foo.hello/1");\n    return 0;\n  }\n}\n'
  local out status
  if out=$(run_nyash_vm -c "$code" --verify 2>&1); then
    status=0
  else
    status=$?
  fi
  if [ "$status" -eq 0 ]; then
    test_fail "expected verifier failure (missing singleton)" "$out"
    return 1
  fi
  if echo "$out" | grep -q "missing static receiver"; then
    test_pass mir_verify_module_function_missing_receiver_vm
  else
    test_fail "verifier message did not mention missing static receiver" "$out"
    return 1
  fi
}

run_test mir_verify_module_function_missing_receiver_vm test_mir_verify_module_function_missing_receiver_vm
exit 0
