#!/bin/bash
# set -eは使わない（個々のテストが失敗しても続行するため）
# string_concat.sh - 文字列結合のテスト

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_simple_concat() {
    local output
    output=$(run_nyash_vm -c 'print("Hello" + " " + "World")' 2>&1)
    check_exact "Hello World" "$output" "simple_concat"
}

test_variable_concat() {
    local script='
local greeting, name, message
greeting = "Hello"
name = "Nyash"
message = greeting + ", " + name + "!"
print(message)
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "Hello, Nyash!" "$output" "variable_concat"
}

test_number_string_concat() {
    local script='
local num, text
num = 42
text = "The answer is " + num
print(text)
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "The answer is 42" "$output" "number_string_concat"
}

# テスト実行
run_test "simple_concat" test_simple_concat
run_test "variable_concat" test_variable_concat
run_test "number_string_concat" test_number_string_concat