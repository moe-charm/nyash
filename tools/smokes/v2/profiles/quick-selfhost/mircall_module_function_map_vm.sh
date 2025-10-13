#!/usr/bin/env bash
# mircall_module_function_map_vm.sh — sanity: ModuleFunction-style path via MIR (MapBox.size)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_module_function_map_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("k", 1)\n    if m.size() != 1 { return 221 }\n    return 0\n  }\n}\n'
  out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_module_function_map_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_module_function_map_vm test_mircall_module_function_map_vm
