#!/bin/bash
# set -eは使わない（個々のテストが失敗しても続行するため）
# basic_print.sh - 基本的なprint機能テスト

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_basic_print() {
    local output
    output=$(run_nyash_vm -c 'print("Hello, World!")')
    compare_outputs "Hello, World!" "$output" "basic_print"
}

# テスト実行
run_test "basic_print" test_basic_print