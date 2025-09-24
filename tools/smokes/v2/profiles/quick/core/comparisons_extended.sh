#!/bin/bash
# comparisons_extended.sh - 比較演算の拡張セット（整数）

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_eq_neq() {
  local script='
if 5 == 5 {
  if 5 != 4 {
    print("OK")
  } else { print("NG") }
} else { print("NG") }
'
  local out; out=$(run_nyash_vm -c "$script" 2>&1)
  check_exact "OK" "$out" "eq_neq"
}

test_le_ge() {
  local script='
if 5 >= 5 {
  if 4 <= 5 {
    print("OK")
  } else { print("NG") }
} else { print("NG") }
'
  local out; out=$(run_nyash_vm -c "$script" 2>&1)
  check_exact "OK" "$out" "le_ge"
}

run_test "eq_neq" test_eq_neq
run_test "le_ge" test_le_ge

