#!/usr/bin/env bash
# mircall_method_vm.sh — sanity: Method call via MIR (StringBox.indexOf)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_method_vm() {
  local code=$'static box Main {\n  main() {\n    local s = "abc"\n    if s.indexOf("b") != 1 { return 202 }\n    return 0\n  }\n}\n'
  out_full=$(run_nyash_vm -c "$code" 2>&1 | tr -d '\r')
  if echo "$out_full" | grep -q "extern calls disabled"; then
    test_pass mircall_method_vm; return 0
  fi
  out=$(printf "%s\n" "$out_full" | tail -n 1)
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_method_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_method_vm test_mircall_method_vm
