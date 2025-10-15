#!/usr/bin/env bash
# extern_c_disallow_symbol_vm.sh — deny-by-default: calling disallowed symbol returns <0

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_extern_c_disallow_symbol_vm() {
  # Ensure allowlist does not include the symbol
  export HAKO_FFI_ALLOW_LIST=""
  local code=$'static box Main {\n  main() {\n    // expect rc < 0 when symbol is not allowed\n    local rc; rc = extern_c "getppid" ();\n    if (rc < 0) { print("OK"); } else { print("NG"); }\n    return rc;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  if echo "$out" | grep -q '^OK$'; then
    test_pass extern_c_disallow_symbol_vm
  else
    test_fail "expected OK (deny)" "exit=$ec out=$out"
    return 1
  fi
}

run_test extern_c_disallow_symbol_vm test_extern_c_disallow_symbol_vm
exit 0

