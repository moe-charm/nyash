#!/bin/bash
# ssot_enabled_disabled_rc_vm.sh — SSOT enabled/disabled should both run OK (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

preflight_plugins || true

test_ssot_enabled_disabled_rc_vm() {
  local code='
    static box Main { main() {
      // trivial: invoke StringBox.size() via vtable/slots
      local s = "hi";
      if s.size() >= 0 { } else { }
      return 0
    }}
  '
  # Run with SSOT disabled
  HAKO_REGISTRY_SSOT_DISABLE=1 run_nyash_vm -c "$code" || return 1
  # Run with SSOT enabled (default)
  run_nyash_vm -c "$code" || return 1
  return 0
}

run_test "ssot_enabled_disabled_rc_vm" test_ssot_enabled_disabled_rc_vm
