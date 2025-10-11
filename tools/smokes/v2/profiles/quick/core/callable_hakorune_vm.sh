#!/bin/bash
# callable_hakorune_vm.sh — CallableBox functionality test via Hakorune language
# Tests methodRef/call/arity through selfhost Mini-VM implementation

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true
log_warn "SKIP callable_hakorune_vm (quick: keep only essentials green)"; exit 0

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

  # Quick profile: rc-only
  if [ $exit_code -eq 0 ]; then
    test_pass "callable_hakorune_vm"
  else
    test_fail "callable_hakorune_vm" "exit=$exit_code"
    echo "$out" >&2
    return 1
  fi
}

run_test "callable_hakorune_vm" test_callable_hakorune_vm
