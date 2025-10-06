#!/bin/bash
# set -eは使わない（個々のテストが失敗しても続行するため）
# variable_assign.sh - 変数宣言と代入のテスト

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_local_variable() {
    local script='
local x
x = 42
print(x)
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
    check_exact "42" "$output" "local_variable"
}

test_string_variable() {
    local script='
local name
name = "Nyash"
print(name)
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
    check_exact "Nyash" "$output" "string_variable"
}

test_multiple_variables() {
    local script='
local a, b, c
a = 1
b = 2
c = 3
print(a + b + c)
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
    check_exact "6" "$output" "multiple_variables"
}

# テスト実行
run_test "local_variable" test_local_variable
run_test "string_variable" test_string_variable
run_test "multiple_variables" test_multiple_variables