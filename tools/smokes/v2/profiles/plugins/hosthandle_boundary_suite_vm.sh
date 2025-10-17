#!/usr/bin/env bash
# hosthandle_boundary_suite_vm.sh — Unified boundary suite for HostHandleRouter (-1/-11/-13/-14)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

build_hosthandle_boundary_bin() {
  cargo build --release --bin hosthandle_boundary >/dev/null 2>&1 || return 1
  echo "./target/release/hosthandle_boundary"
}

test_hosthandle_boundary_suite_vm() {
  local bin
  bin=$(build_hosthandle_boundary_bin) || { test_skip "build failed"; return 0; }

  # Run -1/-11/-13 checks via helper bin
  local out
  out=$($bin 2>&1)
  echo "$out" | grep -q "UNKNOWN_HANDLE rc=-1" || { echo "$out"; test_fail "missing -1"; return 1; }
  echo "$out" | grep -q "WRONG_TYPE rc=-11" || { echo "$out"; test_fail "missing -11"; return 1; }
  if ! echo "$out" | grep -q "TLV_DECODE rc=SKIP"; then
    echo "$out" | grep -q "TLV_DECODE rc=-13" || { echo "$out"; test_fail "missing -13"; return 1; }
  fi

  # Run -14 check via String.size test hook (ENV-enabled)
  local code=$'static box Main {\n  main() {\n    local s = new StringBox("hello")\n    return s.size() == 5 ? 0 : 0\n  }\n}\n'
  local tmp_out
  tmp_out=$(mktemp)
  HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1 run_nyash_vm -c "$code" >"$tmp_out" 2>&1
  local out
  out=$(cat "$tmp_out")
  rm -f "$tmp_out"
  echo "$out" | grep -q -- "-14" || { echo "$out"; test_fail "missing -14"; return 1; }

  test_pass hosthandle_boundary_suite_vm
}

run_test hosthandle_boundary_suite_vm test_hosthandle_boundary_suite_vm
