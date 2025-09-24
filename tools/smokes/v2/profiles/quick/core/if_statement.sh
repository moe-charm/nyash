#!/bin/bash
# set -eは使わない（個々のテストが失敗しても続行するため）
# if_statement.sh - if文のテスト

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_simple_if() {
    local script='
local x
x = 10
if x > 5 {
    print("greater")
} else {
    print("smaller")
}
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "greater" "$output" "simple_if"
}

test_if_else() {
    local script='
local score
score = 75
if score >= 80 {
    print("A")
} elseif score >= 60 {
    print("B")
} else {
    print("C")
}
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "B" "$output" "if_else"
}

test_nested_if() {
    local script='
local a, b
a = 10
b = 20
if a < b {
    if a == 10 {
        print("correct")
    } else {
        print("wrong")
    }
} else {
    print("error")
}
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "correct" "$output" "nested_if"
}

test_if_with_and() {
    local script='
local x, y
x = 5
y = 10
if x > 0 and y > 0 {
    print("both positive")
} else {
    print("not both positive")
}
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "both positive" "$output" "if_with_and"
}

# テスト実行
run_test "simple_if" test_simple_if
run_test "if_else" test_if_else
run_test "nested_if" test_nested_if
run_test "if_with_and" test_if_with_and