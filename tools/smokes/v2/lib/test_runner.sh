#!/bin/bash
# test_runner.sh - 中核実行器（強制使用）
# スモークテストv2の核心ライブラリ

# set -eは使わない（個々のテストが失敗しても全体を続行するため）
set -uo pipefail

# ルート/バイナリ検出（CWDに依存しない実行を保証）
if [ -z "${NYASH_ROOT:-}" ]; then
  export NYASH_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
fi
export NYASH_BIN="${NYASH_BIN:-$NYASH_ROOT/target/release/nyash}"

# グローバル変数
export SMOKES_V2_LIB_LOADED=1
export SMOKES_START_TIME=$(date +%s.%N)
export SMOKES_TEST_COUNT=0
export SMOKES_PASS_COUNT=0
export SMOKES_FAIL_COUNT=0

# 色定義（重複回避）
if [ -z "${RED:-}" ]; then
    readonly RED='\033[0;31m'
    readonly GREEN='\033[0;32m'
    readonly YELLOW='\033[1;33m'
    readonly BLUE='\033[0;34m'
    readonly NC='\033[0m' # No Color
fi

# ログ関数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" >&2
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $*" >&2
}

# 環境チェック（必須）
require_env() {
    local required_tools=("cargo" "grep" "jq")
    local missing_tools=()

    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done

    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Required tools missing: ${missing_tools[*]}"
        log_error "Please install missing tools and try again"
        return 1
    fi

    # Nyash実行ファイル確認
    if [ ! -f "$NYASH_BIN" ]; then
        log_error "Nyash executable not found at $NYASH_BIN"
        log_error "Please run 'cargo build --release' first (in $NYASH_ROOT)"
        return 1
    fi

    log_info "Environment check passed"
    return 0
}

# プラグイン整合性チェック（必須）
preflight_plugins() {
    # プラグインマネージャーが存在する場合は実行
    if [ -f "$(dirname "${BASH_SOURCE[0]}")/plugin_manager.sh" ]; then
        source "$(dirname "${BASH_SOURCE[0]}")/plugin_manager.sh"
        check_plugin_integrity || return 1
    else
        log_warn "Plugin manager not found, skipping plugin checks"
    fi

    return 0
}

# テスト実行関数
run_test() {
    local test_name="$1"
    local test_func="$2"

    ((SMOKES_TEST_COUNT++))
    local start_time=$(date +%s.%N)

    log_info "Running test: $test_name"

    if $test_func; then
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc -l)
        log_success "$test_name (${duration}s)"
        ((SMOKES_PASS_COUNT++))
        return 0
    else
        local end_time=$(date +%s.%N)
        local duration=$(echo "$end_time - $start_time" | bc -l)
        log_error "$test_name (${duration}s)"
        ((SMOKES_FAIL_COUNT++))
        return 1
    fi
}

# Nyash実行ヘルパー（Rust VM）
run_nyash_vm() {
    local program="$1"
    shift
    local USE_PYVM="${SMOKES_USE_PYVM:-0}"
    # -c オプションの場合は一時ファイル経由で実行
    if [ "$program" = "-c" ]; then
        local code="$1"
        shift
        local tmpfile="/tmp/nyash_test_$$.nyash"
        echo "$code" > "$tmpfile"
        # プラグイン初期化メッセージを除外
        NYASH_VM_USE_PY="$USE_PYVM" NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "$NYASH_BIN" "$tmpfile" "$@" 2>&1 | \
            grep -v "^\[UnifiedBoxRegistry\]" | grep -v "^\[FileBox\]" | grep -v "^Net plugin:" | grep -v "^\[.*\] Plugin" | \
            grep -v "Using builtin StringBox" | grep -v "Phase 15.5: Everything is Plugin" | grep -v "cargo build -p nyash-string-plugin" | \
            grep -v "^\[plugin-loader\] backend=" | grep -v "^\[using\] ctx:" | \
            grep -v "^🔌 plugin host initialized" | grep -v "^✅ plugin host fully configured" | \
            grep -v "^🚀 Nyash VM Backend - Executing file:"
        local exit_code=${PIPESTATUS[0]}
        rm -f "$tmpfile"
        return $exit_code
    else
        # プラグイン初期化メッセージを除外
        NYASH_VM_USE_PY="$USE_PYVM" NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "$NYASH_BIN" "$program" "$@" 2>&1 | \
            grep -v "^\[UnifiedBoxRegistry\]" | grep -v "^\[FileBox\]" | grep -v "^Net plugin:" | grep -v "^\[.*\] Plugin" | \
            grep -v "Using builtin StringBox" | grep -v "Phase 15.5: Everything is Plugin" | grep -v "cargo build -p nyash-string-plugin" | \
            grep -v "^\[plugin-loader\] backend=" | grep -v "^\[using\] ctx:" | \
            grep -v "^🔌 plugin host initialized" | grep -v "^✅ plugin host fully configured" | \
            grep -v "^🚀 Nyash VM Backend - Executing file:"
        return ${PIPESTATUS[0]}
    fi
}

# Nyash実行ヘルパー（LLVM）
run_nyash_llvm() {
    local program="$1"
    shift
    # -c オプションの場合は一時ファイル経由で実行
    if [ "$program" = "-c" ]; then
        local code="$1"
        shift
        local tmpfile="/tmp/nyash_test_$$.nyash"
        echo "$code" > "$tmpfile"
        # プラグイン初期化メッセージを除外
        NYASH_VM_USE_PY=0 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "$NYASH_BIN" --backend llvm "$tmpfile" "$@" 2>&1 | \
            grep -v "^\[FileBox\]" | grep -v "^Net plugin:" | grep -v "^\[.*\] Plugin"
        local exit_code=${PIPESTATUS[0]}
        rm -f "$tmpfile"
        return $exit_code
    else
        # プラグイン初期化メッセージを除外
        NYASH_VM_USE_PY=0 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "$NYASH_BIN" --backend llvm "$program" "$@" 2>&1 | \
            grep -v "^\[FileBox\]" | grep -v "^Net plugin:" | grep -v "^\[.*\] Plugin"
        return ${PIPESTATUS[0]}
    fi
}

# シンプルテスト補助（スクリプト互換）
test_pass() { log_success "$1"; return 0; }
test_fail() { log_error "$1 ${2:-}"; return 1; }
test_skip() { log_warn "SKIP $1 ${2:-}"; return 0; }

# 出力比較ヘルパー
compare_outputs() {
    local expected="$1"
    local actual="$2"
    local test_name="$3"

    if [ "$expected" = "$actual" ]; then
        return 0
    else
        log_error "$test_name output mismatch:"
        log_error "  Expected: $expected"
        log_error "  Actual:   $actual"
        return 1
    fi
}

# 結果サマリー出力
print_summary() {
    local end_time=$(date +%s.%N)
    local total_duration=$(echo "$end_time - $SMOKES_START_TIME" | bc -l)

    echo ""
    echo "==============================================="
    echo "Smoke Test Summary"
    echo "==============================================="
    echo "Total tests:  $SMOKES_TEST_COUNT"
    echo "Passed:       $SMOKES_PASS_COUNT"
    echo "Failed:       $SMOKES_FAIL_COUNT"
    echo "Duration:     ${total_duration}s"
    echo ""

    if [ $SMOKES_FAIL_COUNT -eq 0 ]; then
        log_success "All tests passed! ✨"
        return 0
    else
        log_error "$SMOKES_FAIL_COUNT test(s) failed"
        return 1
    fi
}

# JSON出力関数
output_json() {
    local profile="${1:-unknown}"
    local end_time=$(date +%s.%N)
    local total_duration=$(echo "$end_time - $SMOKES_START_TIME" | bc -l)

    cat << EOF
{
  "profile": "$profile",
  "total": $SMOKES_TEST_COUNT,
  "passed": $SMOKES_PASS_COUNT,
  "failed": $SMOKES_FAIL_COUNT,
  "duration": $total_duration,
  "timestamp": "$(date -Iseconds)",
  "success": $([ $SMOKES_FAIL_COUNT -eq 0 ] && echo "true" || echo "false")
}
EOF
}

# JUnit XML出力関数
output_junit() {
    local profile="${1:-unknown}"
    local end_time=$(date +%s.%N)
    local total_duration=$(echo "$end_time - $SMOKES_START_TIME" | bc -l)

    cat << EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="smokes_$profile" tests="$SMOKES_TEST_COUNT" failures="$SMOKES_FAIL_COUNT" time="$total_duration">
  <!-- Individual test cases would be added by specific test scripts -->
</testsuite>
EOF
}
