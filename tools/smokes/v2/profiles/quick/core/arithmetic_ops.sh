#!/bin/bash
# arithmetic_ops.sh - 四則演算のテスト
# set -eは使わない（個々のテストが失敗しても続行するため）

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_addition() {
    local output
    output=$(run_nyash_vm -c 'print(10 + 25)' 2>&1)
    check_exact "35" "$output" "addition"
}

test_subtraction() {
    local output
    output=$(run_nyash_vm -c 'print(100 - 42)' 2>&1)
    check_exact "58" "$output" "subtraction"
}

test_multiplication() {
    local output
    output=$(run_nyash_vm -c 'print(7 * 6)' 2>&1)
    check_exact "42" "$output" "multiplication"
}

test_division() {
    local output
    output=$(run_nyash_vm -c 'print(84 / 2)' 2>&1)
    check_exact "42" "$output" "division"
}

test_complex_expression() {
    local output
    # (10 + 5) * 2 - 8 = 30 - 8 = 22
    output=$(run_nyash_vm -c 'print((10 + 5) * 2 - 8)' 2>&1)
    check_exact "22" "$output" "complex_expression"
}

# テスト実行
run_test "addition" test_addition
run_test "subtraction" test_subtraction
run_test "multiplication" test_multiplication
run_test "division" test_division
run_test "complex_expression" test_complex_expression