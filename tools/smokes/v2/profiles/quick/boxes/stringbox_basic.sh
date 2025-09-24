#!/bin/bash
# set -eは使わない（個々のテストが失敗しても続行するため）
# stringbox_basic.sh - StringBoxの基本操作テスト

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_stringbox_new() {
    local script='
local s
s = new StringBox("Hello")
print(s)
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "Hello" "$output" "stringbox_new"
}

test_stringbox_length() {
    local script='
local s
s = new StringBox("Nyash")
print(s.length())
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "5" "$output" "stringbox_length"
}

test_stringbox_concat() {
    local script='
local s1, s2, result
s1 = new StringBox("Hello")
s2 = new StringBox(" World")
result = s1.concat(s2)
print(result)
'
    local output
    output=$(run_nyash_vm -c "$script" 2>&1)
    check_exact "Hello World" "$output" "stringbox_concat"
}

# テスト実行
run_test "stringbox_new" test_stringbox_new
run_test "stringbox_length" test_stringbox_length
run_test "stringbox_concat" test_stringbox_concat