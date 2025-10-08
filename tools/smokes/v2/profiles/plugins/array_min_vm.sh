#!/bin/bash
# array_min_vm.sh — Plugins suite: minimal ArrayBox ops (push/get/len)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

# If plugins are missing and SKIP mode is enabled, preflight will set skip flag
preflight_plugins || exit 2

test_array_min_vm() {
  local code='static box Main { main() {
    local a = new ArrayBox()
    a.push("x"); a.push("y");
    if a.len() == 2 { print("ok1") } else { print("ng1") }
    if a.get(1) == "y" { print("ok2") } else { print("ng2") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev)
  # Accept either exact ok1/ok2 lines or compare last two lines
  local last2
  last2=$(echo "$out" | tail -n 2 | tr '\n' '|')
  if [[ "$last2" == *"ok1|ok2"* ]]; then
    return 0
  else
    compare_outputs "ok1|ok2" "$last2" "array_min_vm"
  fi
}

run_test "array_min_vm" test_array_min_vm

