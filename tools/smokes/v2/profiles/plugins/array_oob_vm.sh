#!/bin/bash
# array_oob_vm.sh — Plugins suite: ArrayBox OOB get should be safe (policy: null or handled)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

# Preflight: allow SKIP when plugins are not present
preflight_plugins || exit 2

test_array_oob_vm() {
  local code='static box Main { main() {
    local a = new ArrayBox()
    a.push("x"); a.push("y");
    # OOB access should not crash; expected to yield null (policy)
    local v = a.get(5)
    if v == null { print("ok1") } else { print("ng1") }
    if a.len() == 2 { print("ok2") } else { print("ng2") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | filter_noise)
  local last2
  last2=$(echo "$out" | tail -n 2 | tr '\n' '|')
  if [[ "$last2" == *"ok1|ok2"* ]]; then
    return 0
  else
    echo "$out" >&2
    compare_outputs "ok1|ok2" "$last2" "array_oob_vm"
  fi
}

run_test "array_oob_vm" test_array_oob_vm

