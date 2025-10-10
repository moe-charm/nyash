#!/bin/bash
# callable_hakorune_vm.sh — CallableBox functionality test via Hakorune language
# Tests methodRef/call/arity through selfhost Mini-VM implementation

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_callable_hakorune_vm() {
  local test_file="apps/tests/test_callable_direct.hako"

  # Verify test file exists
  if [ ! -f "$test_file" ]; then
    test_fail "callable_hakorune_vm" "Test file not found: $test_file"
    return 1
  fi

  log_info "Running CallableBox tests via Hakorune language"

  # Run test with NYASH_DISABLE_PLUGINS=1 (Rust VM execution)
  out=$(NYASH_DISABLE_PLUGINS=1 run_nyash_vm "$test_file")
  exit_code=$?

  # Debug output (only if SMOKES_DEV_LOG=1)
  if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then
    log_info "Test output:"
    echo "$out" >&2
  fi

  # Check exit code (should be 0 for success)
  if [ $exit_code -ne 0 ]; then
    test_fail "callable_hakorune_vm" "Test execution failed with exit code $exit_code"
    echo "$out" >&2
    return 1
  fi

  # Verify all tests passed
  # Expected output: "✅ All CallableBox tests PASSED! (4/4)"
  echo "$out" | grep -q "All CallableBox tests PASSED" || {
    test_fail "callable_hakorune_vm" "Expected success message not found"
    echo "$out" >&2
    return 1
  }

  # Verify no [FAIL] messages in output
  if echo "$out" | grep -q "\[FAIL\]"; then
    test_fail "callable_hakorune_vm" "Found [FAIL] message in test output"
    echo "$out" >&2
    return 1
  fi

  test_pass "callable_hakorune_vm"
}

run_test "callable_hakorune_vm" test_callable_hakorune_vm
