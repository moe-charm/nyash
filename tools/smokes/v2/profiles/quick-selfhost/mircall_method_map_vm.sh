#!/usr/bin/env bash
# mircall_method_map_vm.sh — sanity: Method call via MIR (MapBox.has/set)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_method_map_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    if m.has("k") != false { return 211 }\n    m.set("k", 1)\n    if m.has("k") != true { return 212 }\n    return 0\n  }\n}\n'
  out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_method_map_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_method_map_vm test_mircall_method_map_vm
