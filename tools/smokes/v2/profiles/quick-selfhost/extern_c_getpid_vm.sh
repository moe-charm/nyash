#!/usr/bin/env bash
# extern_c_getpid_vm.sh — extern_c MVP: getpid()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_extern_c_getpid_vm() {
  local code=$'static box Main {\n  main() {\n    local pid; pid = extern_c "getpid" ();\n    if (pid > 0) { print("OK"); } else { print("NG"); }\n    return pid;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  if echo "$out" | grep -q '^OK$'; then
    test_pass extern_c_getpid_vm
  else
    test_fail "expected OK" "exit=$ec out=$out"
    return 1
  fi
}

run_test extern_c_getpid_vm test_extern_c_getpid_vm
exit 0
