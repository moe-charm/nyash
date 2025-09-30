#!/bin/bash
# plugin_counter_vm.sh - Minimal CounterBox plugin smoke (quick)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_counterbox_inc_get() {
  local script='
  local c, v
  c = new CounterBox()
  c.inc()
  v = c.get()
  print(v)
  '
  local output
  output=$(run_nyash_vm -c "$script" 2>&1 || true)
  # Skip if provider absent or box unknown
  if echo "$output" | grep -qi "Unknown Box type: CounterBox\|plugin host initialized.*backend=stub\|plugins disabled"; then
    test_skip "counterbox_inc_get" "CounterBox plugin not available"
    return 0
  fi
  check_exact "1" "$output" "counterbox_inc_get"
}

run_test "counterbox_inc_get" test_counterbox_inc_get

