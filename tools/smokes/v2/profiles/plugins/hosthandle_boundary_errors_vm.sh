#!/usr/bin/env bash
# hosthandle_boundary_errors_vm.sh — HostHandleRouter boundary rc checks (-1/-11/-13)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_hosthandle_boundary_errors_vm() {
  # Require the helper binary to be built by cargo build --release
  local bin="${NYASH_ROOT}/target/release/hosthandle_boundary"
  if [ ! -x "$bin" ]; then
    test_skip "hosthandle_boundary binary not found (build release first)"; return 0
  fi
  local out
  out="$($bin 2>&1 | filter_noise)"
  # Expect lines with rc codes
  echo "$out" | grep -q "UNKNOWN_HANDLE rc=-1" || { echo "$out"; test_fail "missing rc -1"; return 1; }
  echo "$out" | grep -q "WRONG_TYPE rc=-11" || { echo "$out"; test_fail "missing rc -11"; return 1; }
  # TLV_DECODE may be SKIP in plugin-only builds
  if echo "$out" | grep -q "TLV_DECODE rc=SKIP"; then
    : # acceptable
  else
    echo "$out" | grep -q "TLV_DECODE rc=-13" || { echo "$out"; test_fail "missing rc -13"; return 1; }
  fi
  test_pass hosthandle_boundary_errors_vm
}

run_test hosthandle_boundary_errors_vm test_hosthandle_boundary_errors_vm

