#!/usr/bin/env bash
# extern_c_system_vm.sh — extern_c MVP: system("true") → 0

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_extern_c_system_vm() {
  local code=$'static box Main {\n  main() {\n    local rc; rc = extern_c "system" ("true");\n    if (rc == 0) { print("OK"); } else { print("NG"); }\n    return rc;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  if echo "$out" | grep -q '^OK$'; then
    test_pass extern_c_system_vm
  else
    test_fail "expected OK" "exit=$ec out=$out"
    return 1
  fi
}

run_test extern_c_system_vm test_extern_c_system_vm
exit 0
