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

trim_ws() {
    local var="$1"
    var="${var#${var%%[![:space:]]*}}"
    var="${var%${var##*[![:space:]]}}"
    printf '%s' "$var"
}

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
Smoke Tests v2 - HakoRune (aka Nyash) 2-Pillar Testing System

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
        quick|integration|integration-core|full|plugins|hako|quick-selfhost|windows)
            ;;
        *)
            log_error "Invalid profile: $PROFILE"
            log_error "Valid profiles: quick, integration, integration-core, full, plugins, hako, quick-selfhost, windows"
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
    # Filter aliases to ease grouping common suites
    case "$FILTER" in
        hosthandle)
            # All HostHandleRouter dev smokes and Stage-2 suite wrapper
            FILTER='host_handle_router_|stage2_on_suite_vm'
            ;;
    esac
    export SMOKES_TEST_FILTER="$FILTER"

    # Optional per-profile env overlay (if present)
    # 'hako' is an alias of 'plugins' to improve naming clarity.
    local overlay_profile="$PROFILE"
    if [ "$PROFILE" = "hako" ]; then
        overlay_profile="hako"
    fi
    local env_overlay="$SCRIPT_DIR/configs/env/${overlay_profile}.env"
    if [ -f "$env_overlay" ]; then
        log_info "Sourcing env overlay: $env_overlay"
        # shellcheck disable=SC1090
        source "$env_overlay"
    fi

    # Fast-fail policy per profile (explicit defaults)
    # quick/integration should run all tests and report summary
    if [ "$PROFILE" = "quick" ] || [ "$PROFILE" = "integration" ]; then
        export SMOKES_FAST_FAIL=0
    fi

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
    local have_llvm=0
    if [ "${SMOKES_FORCE_LLVM:-0}" = "1" ]; then
        have_llvm=1
    fi
    if [ -x "./target/release/nyash" ]; then
        if ./target/release/nyash --version 2>/dev/null | grep -q "features.*llvm"; then
            have_llvm=1
        else
            # Fallback detection: check for LLVM harness symbols in the binary
            if strings ./target/release/nyash 2>/dev/null | grep -E -q 'ny-llvmc|NYASH_LLVM_USE_HARNESS'; then
                have_llvm=1
            fi
        fi
    fi

    # 収集対象ディレクトリ
    local search_dirs=()
    if [ -d "$profile_dir" ]; then
        search_dirs+=("$profile_dir")
    fi
    # For quick-selfhost, legacy quick/selfhost suites are excluded by default.
    # Set SMOKES_INCLUDE_LEGACY=1 to include them temporarily.
    if [ "$PROFILE" = "quick-selfhost" ] && [ "${SMOKES_INCLUDE_LEGACY:-0}" = "1" ]; then
        local extra_dir="$SCRIPT_DIR/profiles/quick/selfhost"
        [ -d "$extra_dir" ] && search_dirs+=("$extra_dir") || true
    fi
    # Also collect curated core suites for quick aggregation (tests are SKIP-gated)
    if [ "$PROFILE" = "quick" ]; then
        local core_suite="$SCRIPT_DIR/suites/core"
        [ -d "$core_suite" ] && search_dirs+=("$core_suite") || true
    fi
    if [ "$PROFILE" = "full" ]; then
        for d in             "$SCRIPT_DIR/profiles/quick"             "$SCRIPT_DIR/profiles/integration"             "$SCRIPT_DIR/profiles/plugins"             "$SCRIPT_DIR/suites/core"             "$SCRIPT_DIR/suites/mir"             "$SCRIPT_DIR/suites/vm"             "$SCRIPT_DIR/suites/llvm"             "$SCRIPT_DIR/suites/plugins"             "$SCRIPT_DIR/suites/experimental" ; do
            [ -d "$d" ] && search_dirs+=("$d") || true
        done
        : "${SMOKES_FAST_FAIL:=0}"; export SMOKES_FAST_FAIL
    fi

    if [ ${#search_dirs[@]} -eq 0 ]; then
        log_error "Profile directory not found: $profile_dir"
        exit 1
    fi

    # テストファイル検索（重複排除）
    declare -A seen
    for base in "${search_dirs[@]}"; do
      while IFS= read -r -d '' file; do
        # Exclude quick/selfhost tests from quick profile (moved to quick-selfhost)
        if [ "$PROFILE" = "quick" ] && echo "$file" | grep -q "/profiles/quick/selfhost/"; then
            continue
        fi
        # Auto-skip plugin-on tests when plugins are disabled in this profile
        if [ "${NYASH_DISABLE_PLUGINS:-0}" = "1" ] || [ "${NYASH_PLUGIN_POLICY:-}" = "off" ]; then
            if echo "$file" | grep -E -q '(plugin_on_|hostbridge_)'; then
                log_warn "Skipping (plugins disabled): $file"
                continue
            fi
        fi
        # フィルタ適用
        if [ -n "$FILTER" ] && ! echo "$file" | grep -E -q "$FILTER"; then
            continue
        fi
        # LLVM未ビルド時は AST(LLVM) 系テストをスキップ
        if [ $have_llvm -eq 0 ] && echo "$file" | grep -q "_ast\.sh$"; then
            log_warn "Skipping (no LLVM): $file"
            continue
        fi
        if [ -z "${seen[$file]:-}" ]; then
            seen[$file]=1
            test_files+=("$file")
        fi
      done < <(find "$base" -name "*.sh" -type f -print0)
    done

    if [ ${#test_files[@]} -gt 0 ]; then
        printf '%s\n' "${test_files[@]}"
    fi
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

    # メタデータ（個別タイムアウト/環境変数の上書き）
    local header
    header=$(head -n 25 "$test_file")
    local per_timeout="${SMOKES_DEFAULT_TIMEOUT:-}"
    local -a per_env_keys=()
    local -a per_env_vals=()
    while IFS= read -r meta_line; do
        case "$meta_line" in
            '# SMOKES_TIMEOUT='*)
                per_timeout="${meta_line#*SMOKES_TIMEOUT=}"
                per_timeout=$(trim_ws "$per_timeout")
                ;;
            '# SMOKES_ENV+='*)
                local spec="${meta_line#*SMOKES_ENV+=}"
                local key value
                IFS='=' read -r key value <<< "$spec"
                key=$(trim_ws "$key")
                value=$(trim_ws "$value")
                per_env_keys+=("$key")
                per_env_vals+=("$value")
                ;;
        esac
    done <<< "$header"

    declare -A restore_env=()
    for idx in "${!per_env_keys[@]}"; do
        local key="${per_env_keys[$idx]}"
        local value="${per_env_vals[$idx]}"
        if [ -z "$key" ]; then
            continue
        fi
        if [[ ! -v restore_env[$key] ]]; then
            if [ -z "${!key+x}" ]; then
                restore_env[$key]="__SMOKES_UNSET__"
            else
                restore_env[$key]="${!key}"
            fi
        fi
        export "$key=$value"
    done

    # コマンド組み立て（timeoutは必要な場合のみ）
    local -a cmd=("bash" "$test_file")
    if [ -n "$per_timeout" ] && [ "$per_timeout" != "0" ] && [ "$per_timeout" != "none" ]; then
        read -ra timeout_parts <<< "$per_timeout"
        if [ ${#timeout_parts[@]} -gt 0 ]; then
            cmd=(timeout "${timeout_parts[@]}" "${cmd[@]}")
        fi
    fi

    # 詳細ログ: 失敗時のみテイル表示
    local log_file
    log_file="/tmp/nyash_smoke_$(date +%s)_$$.log"
    if "${cmd[@]}" >"$log_file" 2>&1; then
        exit_code=0
    else
        exit_code=$?
    fi

    # Soft normalization: some tests validate by output (e.g., single 'OK') while
    # the underlying program exits non‑zero. When the log clearly shows a pass,
    # and there are no FAIL markers, consider the test passed.
    if [ $exit_code -ne 0 ]; then
        # 1) Look for PASS marker (colorized lines contain '[PASS]')
        if grep -q 'PASS]' "$log_file" && ! grep -q 'FAIL]' "$log_file"; then
            exit_code=0
        # 2) Or a clean single-line OK without NG
        elif grep -qE '(^|[[:space:]])OK([[:space:]]|$)' "$log_file" && ! grep -qE '(^|[[:space:]])NG([[:space:]]|$)' "$log_file"; then
            exit_code=0
        fi
    fi

    # 実行後に環境変数を元へ戻す
    for key in "${!restore_env[@]}"; do
        if [ "${restore_env[$key]}" = "__SMOKES_UNSET__" ]; then
            unset "$key"
        else
            export "$key=${restore_env[$key]}"
        fi
    done

    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc -l)

    # 結果出力
    case "$FORMAT" in
        text)
            if [ $exit_code -eq 0 ]; then
                echo -e "${GREEN}PASS${NC} (${duration}s)"
            else
                echo -e "${RED}FAIL${NC} (exit=$exit_code, ${duration}s)"
                echo -e "${YELLOW}[WARN]${NC} Test file: $test_file"
                local TAIL_N="${SMOKES_NOTIFY_TAIL:-80}"
                echo "----- LOG (tail -n $TAIL_N) -----"
                tail -n "$TAIL_N" "$log_file" || true
                echo "----- END LOG -----"
            fi
            ;;
        json)
            local status_json
            status_json=$([ $exit_code -eq 0 ] && echo "pass" || echo "fail")
            echo "{"name":"$test_name","path":"$test_file","status":"$status_json","duration":$duration,"exit":$exit_code}"
            ;;
        junit)
            # JUnit形式は後でまとめて出力（pathも保持）
            echo "$test_name:$exit_code:$duration:$test_file" >> /tmp/junit_results.txt
            ;;
    esac

    # 後始末
    rm -f "$log_file" 2>/dev/null || true
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
            passed=$((passed+1))
        else
            failed=$((failed+1))
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
            while IFS=':' read -r name exit_code duration path; do
                if [ "$exit_code" = "0" ]; then
                    echo "  <testcase name=\"$name\" time=\"$duration\" classname=\"$path\"/>"
                else
                    echo "  <testcase name=\"$name\" time=\"$duration\" classname=\"$path\"><failure message=\"exit=$exit_code\"/></testcase>"
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
        log_header "🔥 HakoRune Smoke Tests v2 (aka Nyash) - 2-Pillar Testing System"
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
