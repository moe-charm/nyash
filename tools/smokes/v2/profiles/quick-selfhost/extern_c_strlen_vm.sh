#!/usr/bin/env bash
# extern_c_strlen_vm.sh — extern_c MVP: strlen("hello") → 5

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_extern_c_strlen_vm() {
  local code=$'static box Main {\n  main() {\n    local n; n = extern_c "strlen" ("hello");\n    if (n == 5) { print("OK"); } else { print("NG"); }\n    return n;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  if echo "$out" | grep -q '^OK$'; then
    test_pass extern_c_strlen_vm
  else
    test_fail "expected OK" "exit=$ec out=$out"
    return 1
  fi
}

run_test extern_c_strlen_vm test_extern_c_strlen_vm
exit 0
