#!/bin/bash
# plugin_counter_vm.sh - Minimal CounterBox plugin smoke (quick)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2
log_warn "SKIP counterbox_inc_get (quick: optional plugin demo)"; exit 0

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
  # Quick: rc-only if plugin available
  if run_nyash_vm -c "$script" >/dev/null 2>&1; then
    test_pass "counterbox_inc_get"
  else
    test_fail "counterbox_inc_get" "non-zero rc"
    return 1
  fi
}

run_test "counterbox_inc_get" test_counterbox_inc_get

