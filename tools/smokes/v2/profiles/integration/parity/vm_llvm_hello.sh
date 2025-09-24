#!/bin/bash
# vm_llvm_hello.sh - VM vs LLVM パリティテスト

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_vm_llvm_parity() {
    check_parity -c 'print("Hello from both!")' "vm_llvm_hello_parity"
}

# テスト実行
run_test "vm_llvm_parity" test_vm_llvm_parity
