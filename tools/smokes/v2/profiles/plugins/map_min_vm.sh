#!/bin/bash
# map_min_vm.sh — Plugins suite: minimal MapBox ops (set/get/size/has)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

# If plugins are missing and SKIP mode is enabled, preflight will set skip flag
preflight_plugins || exit 2

test_map_min_vm() {
  local code='static box Main { main() {
    local m = new MapBox()
    m.set("k", "v")
    if m.size() == 1 { print("ok1") } else { print("ng1") }
    if m.get("k") == "v" { print("ok2") } else { print("ng2") }
    if m.has("k") == true { print("ok3") } else { print("ng3") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev)
  # Check last three lines combined
  local last3
  last3=$(echo "$out" | tail -n 3 | tr '\n' '|')
  if [[ "$last3" == *"ok1|ok2|ok3"* ]]; then
    return 0
  else
    compare_outputs "ok1|ok2|ok3" "$last3" "map_min_vm"
  fi
}

run_test "map_min_vm" test_map_min_vm

