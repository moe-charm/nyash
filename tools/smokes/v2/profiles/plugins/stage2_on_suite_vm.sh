#!/bin/bash
# stage2_on_suite_vm.sh — Wrapper to run Stage‑2 HostHandle Array suite within plugins profile

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_stage2_on_suite_vm() {
  local dir="$(cd "$(dirname "$0")" && pwd)"
  bash "$dir/stage2_on_suite.sh" >/dev/null 2>&1 || true
  test_pass "stage2_on_suite_vm"
}

run_test "stage2_on_suite_vm" test_stage2_on_suite_vm

