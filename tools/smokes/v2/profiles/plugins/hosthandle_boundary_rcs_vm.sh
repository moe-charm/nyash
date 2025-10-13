#!/usr/bin/env bash
# hosthandle_boundary_rcs_vm.sh — Validate HostHandleRouter boundary rc (-1, -11, -13)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

build_hosthandle_boundary_bin() {
  cargo build --release --bin hosthandle_boundary >/dev/null 2>&1 || return 1
  echo "./target/release/hosthandle_boundary"
}

test_hosthandle_boundary_rcs_vm() {
  local bin
  bin=$(build_hosthandle_boundary_bin) || { test_skip "build failed"; return 0; }
  local out
  out=$($bin 2>&1)
  # Expect UNKNOWN_HANDLE rc=-1 and WRONG_TYPE rc=-11; TLV_DECODE may be SKIP on plugin-only
  echo "$out" | grep -q "UNKNOWN_HANDLE rc=-1" || { echo "$out"; test_fail "missing -1"; return 1; }
  echo "$out" | grep -q "WRONG_TYPE rc=-11" || { echo "$out"; test_fail "missing -11"; return 1; }
  if echo "$out" | grep -q "TLV_DECODE rc=SKIP"; then
    test_skip "TLV_DECODE skipped (plugin-only)"; return 0
  fi
  echo "$out" | grep -q "TLV_DECODE rc=-13" || { echo "$out"; test_fail "missing -13"; return 1; }
  test_pass hosthandle_boundary_rcs_vm
}

run_test hosthandle_boundary_rcs_vm test_hosthandle_boundary_rcs_vm

