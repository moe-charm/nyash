#!/bin/bash
# map_missing_key_vm.sh — Plugins suite: MapBox missing key semantics (has=false, get=null)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

preflight_plugins || exit 2

test_map_missing_key_vm() {
  local code='static box Main { main() {
    local m = new MapBox()
    if m.has("k") == false { print("ok1") } else { print("ng1") }
    if m.get("k") == null { print("ok2") } else { print("ng2") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | filter_noise)
  sig=$(echo "$out" | grep -E "^(ok1|ok2)$" | tr '\n' '|' )
  if [[ "$sig" == *"ok1|ok2"* ]]; then
    return 0
  else
    echo "$out" >&2
    compare_outputs "ok1|ok2" "$sig" "map_missing_key_vm"
  fi
}

run_test "map_missing_key_vm" test_map_missing_key_vm

