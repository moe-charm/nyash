#!/usr/bin/env bash
# mir_verify_module_function_receiver_mismatch_vm.sh
# Ensure Verifier flags ModuleFunction calls whose receiver is the wrong Box type.

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_verify_module_function_receiver_mismatch_vm() {
  local code=$'static box Foo {\n  hello() { return 0; }\n}\nstatic box Main {\n  main() {\n    local other = new MapBox();\n    call("Foo.hello/0", other);\n    return 0;\n  }\n}\n'
  local out status
  if out=$(run_nyash_vm -c "$code" --verify 2>&1); then
    status=0
  else
    status=$?
  fi
  if [ "$status" -eq 0 ]; then
    test_fail "expected verifier failure (receiver type mismatch)" "$out"
    return 1
  fi
  if echo "$out" | grep -q "receiver type mismatch"; then
    test_pass mir_verify_module_function_receiver_mismatch_vm
  else
    test_fail "verifier message did not mention receiver type mismatch" "$out"
    return 1
  fi
}

run_test mir_verify_module_function_receiver_mismatch_vm test_mir_verify_module_function_receiver_mismatch_vm
exit 0
