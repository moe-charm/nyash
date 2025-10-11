#!/bin/bash
# deterministic_net_denied_vm.sh — deterministic mode denies NET-cap plugin boxes

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_deterministic_net_denied_vm() {
  export HAKO_DETERMINISTIC=1
  export HAKO_PLUGIN_POLICY=${HAKO_PLUGIN_POLICY:-auto}
  local code='
static box Main { main() {
  local s = new ServerBox()
  // Attempt to start to ensure denial
  s.start(8181)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code" )
  local rc=$?
  if [ $rc -ne 0 ] && echo "$out" | grep -Eq "deterministic mode|plugin-on policy forbids builtin fallback"; then
    test_pass "deterministic_net_denied_vm"
  else
    test_fail "expected deterministic denial, got: $(echo "$out" | head -n 1) (rc=$rc)"
  fi
}

run_test "deterministic_net_denied_vm" test_deterministic_net_denied_vm
