#!/bin/bash
# string_methods_vm.sh — Plugins suite: string methods (requires stringbox plugin)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

# If plugins are missing and SKIP mode is enabled, preflight will set skip flag
preflight_plugins || exit 2

test_string_methods_vm() {
  # Simple test that exercises StringBox methods only when plugins available
  # If missing, run_test will SKIP due to SMOKES_SKIP_CUR_TEST=1 flag.
  local code='static box Main { main() {
    local s = "hello"
    if s.substring(1,3) == "el" { print("ok") } else { print("ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev)
  compare_outputs "ok" "$out" "string_methods_vm"
}

run_test "string_methods_vm" test_string_methods_vm
