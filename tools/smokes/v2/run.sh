#!/bin/bash
# run.sh - スモークテストv2 単一エントリポイント
# Usage: ./run.sh --profile {quick|integration|full} [options]

set -euo pipefail

# スクリプトディレクトリ
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# デフォルト値
PROFILE="quick"
FORMAT="text"
JOBS=1
TIMEOUT=""
VERBOSE=false
DRY_RUN=false
FILTER=""
FORCE_CONFIG=""

# カラー定義
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly BOLD='\033[1m'
readonly NC='\033[0m'

# ログ関数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" >&2
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*" >&2
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_header() {
    echo -e "${BOLD}$*${NC}" >&2
}

# ヘルプ表示
show_help() {
    cat << 'EOF'
Smoke Tests v2 - Nyash 2-Pillar Testing System

Usage:
  ./run.sh --profile PROFILE [options]

Profiles:
  quick        Development-time fast checks (1-2 min)
  integration  Basic VM ↔ LLVM parity checks (5-10 min)
  full         Complete matrix testing (15-30 min)

Options:
  --profile PROFILE         Test profile to run
  --filter "PATTERN"        Test filter (e.g., "boxes:string")
  --format FORMAT           Output format: text|json|junit
  --jobs N                  Parallel execution count
  --timeout SEC             Timeout per test in seconds
  --force-config CONFIG     Force configuration: rust_vm_dynamic|llvm_static
  --verbose                 Enable verbose output
  --dry-run                 Show test list without execution
  --help                    Show this help

Examples:
  # Quick development check
  ./run.sh --profile quick

  # Integration with filter
  ./run.sh --profile integration --filter "plugins:*"

  # Full testing with JSON output
  ./run.sh --profile full --format json --jobs 4 --timeout 300

  # Dry run to see what would be tested
  ./run.sh --profile integration --dry-run

Environment Variables:
  SMOKES_FORCE_CONFIG       Force specific configuration
  SMOKES_PLUGIN_MODE        Plugin mode: dynamic|static
  CI                        CI environment detection
EOF
}

# 引数パース
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --profile)
                PROFILE="$2"
                shift 2
                ;;
            --filter)
                FILTER="$2"
                shift 2
                ;;
            --format)
                FORMAT="$2"
                shift 2
                ;;
            --jobs)
                JOBS="$2"
                shift 2
                ;;
            --timeout)
                TIMEOUT="$2"
                shift 2
                ;;
            --force-config)
                FORCE_CONFIG="$2"
                shift 2
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --dry-run)
                DRY_RUN=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    # プロファイル検証
    case "$PROFILE" in
        quick|integration|full)
            ;;
        *)
            log_error "Invalid profile: $PROFILE"
            log_error "Valid profiles: quick, integration, full"
            exit 1
            ;;
    esac

    # フォーマット検証
    case "$FORMAT" in
        text|json|junit)
            ;;
        *)
            log_error "Invalid format: $FORMAT"
            log_error "Valid formats: text, json, junit"
            exit 1
            ;;
    esac
}

# 環境設定
setup_environment() {
    log_info "Setting up environment for profile: $PROFILE"

    # 共通ライブラリ読み込み
    source "$SCRIPT_DIR/lib/test_runner.sh"
    source "$SCRIPT_DIR/lib/plugin_manager.sh"
    source "$SCRIPT_DIR/lib/result_checker.sh"
    source "$SCRIPT_DIR/lib/preflight.sh"

    # 設定読み込み
    if [ -n "$FORCE_CONFIG" ]; then
        export SMOKES_FORCE_CONFIG="$FORCE_CONFIG"
        log_info "Forced configuration: $FORCE_CONFIG"
    fi

    source "$SCRIPT_DIR/configs/auto_detect.conf"
    auto_configure "$PROFILE"

    # プロファイル専用設定
    export SMOKES_CURRENT_PROFILE="$PROFILE"

    # コマンドライン引数の環境変数設定
    if [ -n "$TIMEOUT" ]; then
        export SMOKES_DEFAULT_TIMEOUT="$TIMEOUT"
    fi

    if [ "$VERBOSE" = true ]; then
        export NYASH_CLI_VERBOSE=1
        export SMOKES_LOG_LEVEL="debug"
    fi

    export SMOKES_PARALLEL_JOBS="$JOBS"
    export SMOKES_OUTPUT_FORMAT="$FORMAT"
    export SMOKES_TEST_FILTER="$FILTER"

    # 作業ディレクトリ移動（Nyashプロジェクトルートへ）
    cd "$SCRIPT_DIR/../../.."
    log_info "Working directory: $(pwd)"
}

# プリフライトチェック
run_preflight() {
    log_info "Running preflight checks..."

    if ! preflight_all; then
        log_error "Preflight checks failed"
        log_error "Run with --verbose for detailed information"
        exit 1
    fi

    log_success "Preflight checks passed"
}

# テストファイル検索
find_test_files() {
    local profile_dir="$SCRIPT_DIR/profiles/$PROFILE"
    local test_files=()

    if [ ! -d "$profile_dir" ]; then
        log_error "Profile directory not found: $profile_dir"
        exit 1
    fi

    # テストファイル検索
    while IFS= read -r -d '' file; do
        # フィルタ適用
        if [ -n "$FILTER" ]; then
            local relative_path
            relative_path=$(realpath --relative-to="$profile_dir" "$file")
            if ! echo "$relative_path" | grep -q "$FILTER"; then
                continue
            fi
        fi
        test_files+=("$file")
    done < <(find "$profile_dir" -name "*.sh" -type f -print0)

    printf '%s\n' "${test_files[@]}"
}

# 単一テスト実行
run_single_test() {
    local test_file="$1"
    local test_name
    test_name=$(basename "$test_file" .sh)

    if [ "$FORMAT" = "text" ]; then
        echo -n "Running $test_name... "
    fi

    local start_time end_time duration exit_code
    start_time=$(date +%s.%N)

    # タイムアウト付きテスト実行
    local timeout_cmd=""
    if [ -n "${SMOKES_DEFAULT_TIMEOUT:-}" ]; then
        timeout_cmd="timeout ${SMOKES_DEFAULT_TIMEOUT}"
    fi

    if $timeout_cmd bash "$test_file" >/dev/null 2>&1; then
        exit_code=0
    else
        exit_code=$?
    fi

    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc -l)

    # 結果出力
    case "$FORMAT" in
        text)
            if [ $exit_code -eq 0 ]; then
                echo -e "${GREEN}PASS${NC} (${duration}s)"
            else
                echo -e "${RED}FAIL${NC} (${duration}s)"
            fi
            ;;
        json)
            echo "{\"name\":\"$test_name\",\"status\":\"$([ $exit_code -eq 0 ] && echo "pass" || echo "fail")\",\"duration\":$duration}"
            ;;
        junit)
            # JUnit形式は後でまとめて出力
            echo "$test_name:$exit_code:$duration" >> /tmp/junit_results.txt
            ;;
    esac

    return $exit_code
}

# テスト実行
run_tests() {
    local test_files
    mapfile -t test_files < <(find_test_files)

    if [ ${#test_files[@]} -eq 0 ]; then
        log_warn "No test files found for profile: $PROFILE"
        if [ -n "$FILTER" ]; then
            log_warn "Filter applied: $FILTER"
        fi
        exit 0
    fi

    log_info "Found ${#test_files[@]} test files"

    # Dry run
    if [ "$DRY_RUN" = true ]; then
        log_header "Test files that would be executed:"
        for file in "${test_files[@]}"; do
            echo "  $(realpath --relative-to="$SCRIPT_DIR" "$file")"
        done
        exit 0
    fi

    # テスト実行開始
    log_header "Starting $PROFILE profile tests"

    local passed=0
    local failed=0
    local start_time
    start_time=$(date +%s.%N)

    # JSON形式の場合はヘッダー出力
    if [ "$FORMAT" = "json" ]; then
        echo '{"profile":"'$PROFILE'","tests":['
    fi

    # JUnit用ファイル初期化
    if [ "$FORMAT" = "junit" ]; then
        echo -n > /tmp/junit_results.txt
    fi

    # テスト実行
    local first_test=true
    for test_file in "${test_files[@]}"; do
        if [ "$FORMAT" = "json" ] && [ "$first_test" = false ]; then
            echo ","
        fi
        first_test=false

        if run_single_test "$test_file"; then
            ((passed++))
        else
            ((failed++))
            # Fast fail モード
            if [ "${SMOKES_FAST_FAIL:-0}" = "1" ]; then
                log_warn "Fast fail enabled, stopping on first failure"
                break
            fi
        fi
    done

    # 結果出力
    local end_time total_duration
    end_time=$(date +%s.%N)
    total_duration=$(echo "$end_time - $start_time" | bc -l)

    case "$FORMAT" in
        text)
            echo ""
            log_header "Test Results Summary"
            echo "Profile: $PROFILE"
            echo "Total: $((passed + failed))"
            echo "Passed: $passed"
            echo "Failed: $failed"
            echo "Duration: ${total_duration}s"

            if [ $failed -eq 0 ]; then
                log_success "All tests passed! ✨"
            else
                log_error "$failed test(s) failed"
            fi
            ;;
        json)
            echo '],"summary":{"total":'$((passed + failed))',"passed":'$passed',"failed":'$failed',"duration":'$total_duration'}}'
            ;;
        junit)
            cat << EOF
<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="smokes_$PROFILE" tests="$((passed + failed))" failures="$failed" time="$total_duration">
EOF
            while IFS=':' read -r name exit_code duration; do
                if [ "$exit_code" = "0" ]; then
                    echo "  <testcase name=\"$name\" time=\"$duration\"/>"
                else
                    echo "  <testcase name=\"$name\" time=\"$duration\"><failure message=\"Test failed\"/></testcase>"
                fi
            done < /tmp/junit_results.txt
            echo "</testsuite>"
            rm -f /tmp/junit_results.txt
            ;;
    esac

    # 終了コード
    [ $failed -eq 0 ]
}

# メイン処理
main() {
    # 引数パース
    parse_arguments "$@"

    # バナー表示
    if [ "$FORMAT" = "text" ]; then
        log_header "🔥 Nyash Smoke Tests v2 - 2-Pillar Testing System"
        log_info "Profile: $PROFILE | Format: $FORMAT | Jobs: $JOBS"
        if [ -n "$FILTER" ]; then
            log_info "Filter: $FILTER"
        fi
        echo ""
    fi

    # 環境設定
    setup_environment

    # プリフライト
    run_preflight

    # テスト実行
    run_tests
}

# エラーハンドリング
trap 'log_error "Script interrupted"; exit 130' INT TERM

# メイン実行
main "$@"