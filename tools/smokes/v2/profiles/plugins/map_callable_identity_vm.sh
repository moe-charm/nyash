#!/bin/bash
# map_callable_identity_vm.sh — Plugins: Map stores CallableBox and preserves identity

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_map_callable_identity_vm() {
  local code='static box Main { main() {
    local arr = new ArrayBox()
    arr.push(1)
    local cb = arr.methodRef("push", 1)
    local args = new ArrayBox()
    args.push(99)
    local m = new MapBox()
    m.set("cb", cb)
    local cb2 = m.get("cb")
    cb2.call(args)
    if arr.size() == 2 { print("call-ok") } else { print("call-ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local last; last=$(echo "$out" | tail -n 1)
  if [[ "$last" == "call-ok" ]]; then
    return 0
  fi
  compare_outputs "call-ok" "$last" "map_callable_identity_vm"
}

run_test "map_callable_identity_vm" test_map_callable_identity_vm
