#!/bin/bash
# plugin_map_min_vm.sh - Minimal MapBox smoke (quick)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_mapbox_min_ops() {
  # Prefer plugin path if available; builtin path also acceptable
  local script='
  local m, sz, v
  m = new MapBox()
  m.set("k", "v")
  sz = m.size()
  v = m.get("k")
  print(sz)
  print(v)
  '
  local output
  output=$(NYASH_VM_PLUGIN_PREFER_MAP=1 NYASH_CLI_VERBOSE=0 run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
  # Expect size then value
  local last2
  last2=$(echo "$output" | tail -n 2 | tr '\n' '|')
  if [[ "$last2" == *"1|v"* ]]; then
    test_pass "mapbox_min_ops"
  else
    compare_outputs "1|v" "$last2" "mapbox_min_ops"
  fi
}

run_test "mapbox_min_ops" test_mapbox_min_ops
