#!/bin/bash
# result_checker.sh - 結果検証
# 出力比較、パリティテスト、回帰検証の統一システム

# set -eは使わない（個々のテストが失敗しても全体を続行するため）
set -uo pipefail
# Guard against double sourcing
if [ -n "${RESULT_CHECKER_SH_LOADED:-}" ]; then
    return 0 2>/dev/null || true
fi
RESULT_CHECKER_SH_LOADED=1

# 結果比較種別
readonly EXACT_MATCH="exact"
readonly REGEX_MATCH="regex"
readonly NUMERIC_RANGE="numeric"
readonly JSON_MATCH="json"

# 結果チェッカー：完全一致
check_exact() {
    local expected="$1"
    local actual="$2"
    local test_name="${3:-unknown}"

    if [ "$expected" = "$actual" ]; then
        return 0
    else
        echo "[FAIL] $test_name: Exact match failed" >&2
        echo "  Expected: '$expected'" >&2
        echo "  Actual:   '$actual'" >&2
        return 1
    fi
}

# 結果チェッカー：正規表現マッチ
check_regex() {
    local pattern="$1"
    local actual="$2"
    local test_name="${3:-unknown}"

    if echo "$actual" | grep -qE "$pattern"; then
        return 0
    else
        echo "[FAIL] $test_name: Regex match failed" >&2
        echo "  Pattern:  '$pattern'" >&2
        echo "  Actual:   '$actual'" >&2
        return 1
    fi
}

# 結果チェッカー：数値範囲
check_numeric_range() {
    local min="$1"
    local max="$2"
    local actual="$3"
    local test_name="${4:-unknown}"

    # 数値抽出（最初の数値を取得）
    local number
    number=$(echo "$actual" | grep -oE '[0-9]+(\.[0-9]+)?' | head -n1)

    if [ -z "$number" ]; then
        echo "[FAIL] $test_name: No number found in output" >&2
        echo "  Actual: '$actual'" >&2
        return 1
    fi

    # bc を使用した数値比較
    if echo "$number >= $min && $number <= $max" | bc -l | grep -q "1"; then
        return 0
    else
        echo "[FAIL] $test_name: Number out of range" >&2
        echo "  Range:    [$min, $max]" >&2
        echo "  Actual:   $number" >&2
        return 1
    fi
}

# 結果チェッカー：JSON比較
check_json() {
    local expected_json="$1"
    local actual_json="$2"
    local test_name="${3:-unknown}"

    # JSONパース可能性チェック
    if ! echo "$expected_json" | jq . >/dev/null 2>&1; then
        echo "[FAIL] $test_name: Expected JSON is invalid" >&2
        return 1
    fi

    if ! echo "$actual_json" | jq . >/dev/null 2>&1; then
        echo "[FAIL] $test_name: Actual JSON is invalid" >&2
        echo "  Actual: '$actual_json'" >&2
        return 1
    fi

    # JSON正規化比較
    local expected_normalized actual_normalized
    expected_normalized=$(echo "$expected_json" | jq -c -S .)
    actual_normalized=$(echo "$actual_json" | jq -c -S .)

    if [ "$expected_normalized" = "$actual_normalized" ]; then
        return 0
    else
        echo "[FAIL] $test_name: JSON comparison failed" >&2
        echo "  Expected: $expected_normalized" >&2
        echo "  Actual:   $actual_normalized" >&2
        return 1
    fi
}

# パリティテスト：VM vs LLVM比較
check_parity() {
    local program="$1"
    local code=""
    local test_name
    local timeout

    if [ "$program" = "-c" ]; then
        code="$2"
        test_name="${3:-parity_test}"
        timeout="${4:-30}"
    else
        test_name="${2:-parity_test}"
        timeout="${3:-30}"
    fi

    local vm_output llvm_output vm_exit llvm_exit

    # Local noise filter for parity runs (applies to both VM/LLVM)
    # Prefer shared filter_noise() if loaded from test_runner.sh
    parity_filter_noise() {
        if type filter_noise >/dev/null 2>&1; then
            filter_noise
            return
        fi
        # Minimal built-in fallback
        grep -v '^\[UnifiedBoxRegistry\]' \
        | grep -v '^\[FileBox\]' \
        | grep -v '^Net plugin:' \
        | grep -v '^\[.*\] Plugin' \
        | grep -v '^\[using\]' \
        | grep -v '^\[using/resolve\]' \
        | grep -v '^\[deprecate\] CLI name' \
        | grep -v '^Exception ignored in:' \
        | grep -v '^Traceback \(most recent call last\):' \
        | grep -v 'llvmlite/binding/ffi\.py' \
        | grep -v "FunctionPassManager object has no attribute '_as_parameter_'" \
        | grep -v '^\[env\] NYASH_ENABLE_USING is deprecated; use NYASH_USING instead' \
        | grep -v '^\[deprecate\] \[modules\.aliases\]' \
        | grep -v '^🔧 Mock LLVM Backend Execution' \
        | grep -v '^✅ Mock exit code:' \
        | sed -E 's/^❌[[:space:]]*//' \
        | sed -E 's/^Pipeline error: *//' \
        | sed -E 's/\bbb[0-9]+\b/bb<ID>/g'
    }

    # Rust VM 実行
    if [ "$program" = "-c" ]; then
        if vm_output=$(timeout "$timeout" bash -c "NYASH_DISABLE_PLUGINS=1 ./target/release/nyash -c \"$code\" 2>&1"); then
            vm_exit=0
        else
            vm_exit=$?
        fi
    else
        if vm_output=$(timeout "$timeout" bash -c "NYASH_DISABLE_PLUGINS=1 ./target/release/nyash \"$program\" 2>&1"); then
            vm_exit=0
        else
            vm_exit=$?
        fi
    fi

    # LLVM（Pythonハーネス）実行
    if [ "$program" = "-c" ]; then
        if llvm_output=$(timeout "$timeout" bash -c "PYTHONPATH=\"${PYTHONPATH:-$NYASH_ROOT}\" NYASH_NY_LLVM_COMPILER=\"${NYASH_NY_LLVM_COMPILER:-$NYASH_ROOT/target/release/ny-llvmc}\" NYASH_EMIT_EXE_NYRT=\"${NYASH_EMIT_EXE_NYRT:-$NYASH_ROOT/target/release}\" NYASH_LLVM_USE_HARNESS=1 NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend llvm -c \"$code\" 2>&1"); then
            llvm_exit=0
        else
            llvm_exit=$?
        fi
    else
        if llvm_output=$(timeout "$timeout" bash -c "PYTHONPATH=\"${PYTHONPATH:-$NYASH_ROOT}\" NYASH_NY_LLVM_COMPILER=\"${NYASH_NY_LLVM_COMPILER:-$NYASH_ROOT/target/release/ny-llvmc}\" NYASH_EMIT_EXE_NYRT=\"${NYASH_EMIT_EXE_NYRT:-$NYASH_ROOT/target/release}\" NYASH_LLVM_USE_HARNESS=1 NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend llvm \"$program\" 2>&1"); then
            llvm_exit=0
        else
            llvm_exit=$?
        fi
    fi

    # 終了コード比較
    if [ "$vm_exit" != "$llvm_exit" ]; then
        echo "[FAIL] $test_name: Exit code mismatch" >&2
        echo "  VM exit:   $vm_exit" >&2
        echo "  LLVM exit: $llvm_exit" >&2
        return 1
    fi

    # 出力比較（正規化＋ノイズ除去）
    local vm_normalized llvm_normalized
    vm_normalized=$(echo "$vm_output" | parity_filter_noise | sed 's/[[:space:]]*$//' | sort)
    llvm_normalized=$(echo "$llvm_output" | parity_filter_noise | sed 's/[[:space:]]*$//' | sort)

    if [ "$vm_normalized" = "$llvm_normalized" ]; then
        echo "[PASS] $test_name: VM ↔ LLVM parity verified" >&2
        return 0
    else
        echo "[FAIL] $test_name: VM ↔ LLVM output mismatch" >&2
        echo "  VM output:" >&2
        echo "$vm_output" | sed 's/^/    /' >&2
        echo "  LLVM output:" >&2
        echo "$llvm_output" | sed 's/^/    /' >&2
        return 1
    fi
}

# 性能比較テスト
check_performance() {
    local program="$1"
    local max_duration="$2"
    local test_name="${3:-performance_test}"

    local start_time end_time duration

    start_time=$(date +%s.%N)

    if NYASH_DISABLE_PLUGINS=1 ./target/release/nyash "$program" >/dev/null 2>&1; then
        end_time=$(date +%s.%N)
        duration=$(echo "$end_time - $start_time" | bc -l)

        if echo "$duration <= $max_duration" | bc -l | grep -q "1"; then
            echo "[PASS] $test_name: Performance OK (${duration}s <= ${max_duration}s)" >&2
            return 0
        else
            echo "[FAIL] $test_name: Performance too slow (${duration}s > ${max_duration}s)" >&2
            return 1
        fi
    else
        echo "[FAIL] $test_name: Execution failed" >&2
        return 1
    fi
}

# エラーパターン検証
check_error_pattern() {
    local program="$1"
    local expected_error_pattern="$2"
    local test_name="${3:-error_test}"

    local output exit_code

    if output=$(./target/release/nyash "$program" 2>&1); then
        exit_code=0
    else
        exit_code=$?
    fi

    # エラーが期待される場合
    if [ "$exit_code" -eq 0 ]; then
        echo "[FAIL] $test_name: Expected error but execution succeeded" >&2
        echo "  Output: '$output'" >&2
        return 1
    fi

    # エラーパターンマッチ
    if echo "$output" | grep -qE "$expected_error_pattern"; then
        echo "[PASS] $test_name: Expected error pattern found" >&2
        return 0
    else
        echo "[FAIL] $test_name: Error pattern not matched" >&2
        echo "  Expected pattern: '$expected_error_pattern'" >&2
        echo "  Actual output:    '$output'" >&2
        return 1
    fi
}

# 汎用チェッカー（種別自動判定）
check_result() {
    local check_type="$1"
    shift

    case "$check_type" in
        "$EXACT_MATCH")
            check_exact "$@"
            ;;
        "$REGEX_MATCH")
            check_regex "$@"
            ;;
        "$NUMERIC_RANGE")
            check_numeric_range "$@"
            ;;
        "$JSON_MATCH")
            check_json "$@"
            ;;
        "parity")
            check_parity "$@"
            ;;
        "performance")
            check_performance "$@"
            ;;
        "error")
            check_error_pattern "$@"
            ;;
        *)
            echo "[ERROR] Unknown check type: $check_type" >&2
            return 1
            ;;
    esac
}

# 使用例とヘルプ
show_result_checker_help() {
    cat << 'EOF'
Result Checker for Smoke Tests v2

Usage:
  source lib/result_checker.sh

Functions:
  check_exact <expected> <actual> [test_name]
  check_regex <pattern> <actual> [test_name]
  check_numeric_range <min> <max> <actual> [test_name]
  check_json <expected_json> <actual_json> [test_name]
  check_parity <program> [test_name] [timeout]
  check_performance <program> <max_duration> [test_name]
  check_error_pattern <program> <error_pattern> [test_name]
  check_result <type> <args...>

Examples:
  # 完全一致
  check_exact "Hello" "$output" "greeting_test"

  # 正規表現
  check_regex "^[0-9]+$" "$output" "number_test"

  # 数値範囲
  check_numeric_range 1 100 "$output" "score_test"

  # パリティテスト
  check_parity "test.nyash" "vm_llvm_parity"

  # 性能テスト
  check_performance "benchmark.nyash" 5.0 "speed_test"
EOF
}
