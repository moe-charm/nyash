#!/bin/bash
# deterministic_filebox_denied_vm.sh — deterministic mode denies IO-cap plugin boxes

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_deterministic_filebox_denied_vm() {
  export HAKO_DETERMINISTIC=1
  export HAKO_PLUGIN_POLICY=${HAKO_PLUGIN_POLICY:-auto}
  local code='
static box Main { main() {
  local f = new FileBox()
  return 0
}}
'
  out=$(run_nyash_vm -c "$code" --dev)
  if echo "$out" | grep -Eq "deterministic mode|plugin-on policy forbids builtin fallback"; then
    test_pass "deterministic_filebox_denied_vm"
  else
    test_fail "expected deterministic denial, got: $(echo "$out" | head -n 1)"
  fi
}

run_test "deterministic_filebox_denied_vm" test_deterministic_filebox_denied_vm
