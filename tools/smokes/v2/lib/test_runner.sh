#!/bin/bash
# test_runner.sh - 中核実行器（強制使用）
# スモークテストv2の核心ライブラリ

# set -eは使わない（個々のテストが失敗しても全体を続行するため）
set -uo pipefail

alias_env_prefixes() {
  # Map HAKO_/HAKU_/HRN_ → NYASH_* when NYASH_* is unset (non-destructive)
  while IFS='=' read -r k v; do
    case "$k" in
      HAKO_*|HAKU_*|HRN_*)
        tail="${k#HAKO_}"; if [ "$tail" = "$k" ]; then tail="${k#HAKU_}"; fi; if [ "$tail" = "$k" ]; then tail="${k#HRN_}"; fi
        ny="NYASH_${tail}"
        if [ -z "${!ny:-}" ]; then export "$ny"="$v"; fi ;;
    esac
  done < <(env)
}

# ルート/バイナリ検出（CWDに依存しない実行を保証）
alias_env_prefixes
if [ -z "${NYASH_ROOT:-}" ]; then
  # Prefer HAKO_ROOT as an alias when present
  if [ -n "${HAKO_ROOT:-}" ]; then export NYASH_ROOT="$HAKO_ROOT"; else
    export NYASH_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
  fi
fi
# Optional profile env overlay (e.g., SMOKES_PROFILE_ENV=quick)
if [ -n "${SMOKES_PROFILE_ENV:-}" ]; then
  CAND="$NYASH_ROOT/tools/smokes/v2/configs/${SMOKES_PROFILE_ENV}.env"
  if [ -f "$CAND" ]; then
    # shellcheck disable=SC1090
    source "$CAND"
  fi
fi
# Ensure using resolver is enabled by default for smokes unless explicitly disabled
if [ -z "${NYASH_USING:-}" ] && [ -z "${NYASH_ENABLE_USING:-}" ]; then
  export NYASH_USING=1
fi
# Provide a minimal default modules mapping for selfhost Mini-VM smokes when unset
if [ -z "${NYASH_MODULES:-}" ]; then
  export NYASH_MODULES="selfhost.vm.mir_min=apps/selfhost/vm/boxes/mir_vm_min.hako"
fi
# Dev-friendly VM tolerance to avoid hard-stopping on Void during bring-up
if [ -z "${NYASH_VM_TOLERATE_VOID:-}" ]; then
  export NYASH_VM_TOLERATE_VOID=0
fi
# VM fuel/limits for smokes: enable conservative caps unless explicitly set by caller.
if [ -z "${NYASH_VM_MAX_INSTRUCTIONS:-}" ]; then
  export NYASH_VM_MAX_INSTRUCTIONS="${SMOKES_VM_MAX_INSTRUCTIONS:-1000000}"
fi

# Default: enable callable async for smokes unless explicitly set
if [ -z "${HAKO_CALLABLE_ASYNC:-}" ] && [ -z "${NYASH_CALLABLE_ASYNC:-}" ]; then
  export HAKO_CALLABLE_ASYNC=1
fi
if [ -z "${NYASH_VM_MAX_BLOCK_EXEC:-}" ]; then
  export NYASH_VM_MAX_BLOCK_EXEC="${SMOKES_VM_MAX_BLOCK_EXEC:-200000}"
fi
# Optional dev logs: enable resolver trace when requested
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then
  export NYASH_RESOLVE_TRACE=1
fi
# Binary path detection: prefer hakorune → hako → nyash unless NYASH_BIN is set
if [ -z "${NYASH_BIN:-}" ]; then
  for cand in "$NYASH_ROOT/target/release/hakorune" "$NYASH_ROOT/target/release/hako" "$NYASH_ROOT/target/release/nyash"; do
    if [ -x "$cand" ]; then NYASH_BIN="$cand"; break; fi
  done
  export NYASH_BIN="${NYASH_BIN:-$NYASH_ROOT/target/release/nyash}"
fi

# グローバル変数
export SMOKES_V2_LIB_LOADED=1
export SMOKES_START_TIME=$(date +%s.%N)
export SMOKES_TEST_COUNT=0
export SMOKES_PASS_COUNT=0
export SMOKES_FAIL_COUNT=0

# 補助ライブラリの読込み（結果検証など）
_SMOKES_LIB_DIR="$(dirname "${BASH_SOURCE[0]}")"
if [ -f "${_SMOKES_LIB_DIR}/result_checker.sh" ]; then
  # shellcheck disable=SC1090
  source "${_SMOKES_LIB_DIR}/result_checker.sh"
fi

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

# 共通ノイズフィルタ（VM実行時の出力整形）
filter_noise() {
    # プラグイン初期化やメタログ、動的ローダの案内等を除去
    grep -v "^\[UnifiedBoxRegistry\]" \
      | grep -v "^\[FileBox\]" \
      | grep -v "^Net plugin:" \
  | grep -v "^\[.*\] Plugin" \
      | grep -v "^\[using\]" \
      | grep -v "^\[using/resolve\]" \
      | grep -v "^\[builder\]" \
      | grep -v '^\[TypeRegistry\] CORE type ids:' \
      | grep -v "^\\[vm-trace\\]" \
  | grep -v '^\{"ev":' \
      | grep -v '^\[warn\] dev fallback: user instance BoxCall' \
      | grep -v '^\[deps\] missing:' \
      | sed -E 's/^❌ VM fallback error: *//' \
      | sed -E 's/^❌ Pipeline error: *//' \
  | sed -E 's/^VM execution error: VM fallback error: *//' \
  | grep -v '^VM execution error: ' \
  | grep -v '^\[DEBUG-' \
  | grep -v '^Exception ignored in:' \
  | grep -v '^Traceback \(most recent call last\):' \
  | grep -v 'llvmlite/binding/ffi\.py' \
  | grep -v "FunctionPassManager object has no attribute '_as_parameter_'" \
  | grep -v '^\[warn\] dev verify:' \
  | grep -v '^Invalid instruction: operation on unborn instance (call birth() first)$' \
  | grep -v '^\[warn\] dev verify: NewBox ' \
  | grep -v '^\[warn\] dev verify: NewBox→birth invariant warnings:' \
  | grep -v '^\{"kind":"contracts_' \
  | grep -v '^\{"resolve":' \
  | grep -v '^\[deprecate\] CLI name' \
  | grep -v '^\[env\] NYASH_ENABLE_USING is deprecated; use NYASH_USING instead' \
  | grep -v '^\[deprecate\] \[modules.aliases\]' \
  | grep -v '^\[deprecate\]' \
  | grep -v "plugins/nyash-array-plugin" \
  | grep -v "plugins/nyash-map-plugin" \
      | grep -v "Phase 15.5: Everything is Plugin" \
      | grep -v "cargo build -p nyash-string-plugin" \
      | grep -v "^\[plugin-loader\] backend=" \
      | grep -v "^\[using\] ctx:" \
      | grep -v "^🔌 plugin host initialized" \
      | grep -v "^✅ plugin host fully configured" \
      | grep -v "Failed to load nyash.toml - plugins disabled" \
      | grep -v "^🚀 Nyash VM Backend - Executing file:" \
      | grep -v '^🔧 Mock LLVM Backend Execution' \
      | grep -v '^✅ Mock exit code:' \
  | sed -E 's/^❌[[:space:]]*//' \
  | sed -E 's/^Pipeline error: *//' \
  | sed -E 's/bb[0-9]+/bb<ID>/g' \
  | grep -v '^sed: -e expression' \
  | grep -v -E '^sed: .*unterminated .s. command' \
  | grep -v -E '^sed: .*コマンドが終了していません'
}

# Ensure hako.toml exists when tests generate only nyash.toml (compat copy)
ensure_hako_toml() {
    local dirs=("." "$NYASH_ROOT")
    for d in "${dirs[@]}"; do
        if [ -f "$d/nyash.toml" ] && [ ! -f "$d/hako.toml" ]; then
            cp -f "$d/nyash.toml" "$d/hako.toml" 2>/dev/null || true
        fi
    done
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

    # 実行ファイル確認
    if [ ! -f "$NYASH_BIN" ]; then
    log_error "Executable not found at $NYASH_BIN (tried hakorune/hako/nyash)"
    log_error "Please run 'cargo build --release' first (in $NYASH_ROOT)"
        return 1
    fi

    log_info "Environment check passed"
    return 0
}

# プラグイン整合性チェック（必須）
preflight_plugins() {
    # Allow tests to opt-out explicitly
    if [ "${SMOKES_DISABLE_PLUGIN_CHECKS:-0}" = "1" ]; then
        log_warn "Plugin checks disabled for this smoke (SMOKES_DISABLE_PLUGIN_CHECKS=1)"
        return 0
    fi
    # Skip when plugins are explicitly disabled by environment
    if [ "${NYASH_DISABLE_PLUGINS:-0}" = "1" ] || [ "${NYASH_PLUGIN_POLICY:-}" = "off" ]; then
        log_warn "Plugins disabled via env; skipping plugin checks"
        return 0
    fi
    # プラグインマネージャーが存在する場合は実行
    if [ -f "$(dirname "${BASH_SOURCE[0]}")/plugin_manager.sh" ]; then
        source "$(dirname "${BASH_SOURCE[0]}")/plugin_manager.sh"
        # Ensure plugin env (policy/config) is applied before integrity checks
        setup_plugin_env || true
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
    if [ "${SMOKES_ENV_STAMP:-0}" = "1" ]; then
        print_env_stamp
    fi

    # Optional: plugin-missing skip gate (set by preflight_plugins)
    if [ "${SMOKES_SKIP_CUR_TEST:-0}" = "1" ]; then
        log_warn "SKIP $test_name (${SMOKES_SKIP_REASON:-plugins missing})"
        # reset skip flag for next test
        unset SMOKES_SKIP_CUR_TEST || true
        unset SMOKES_SKIP_REASON || true
        return 0
    fi

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
    local EXTRA_ARGS=()
    if [ "${SMOKES_USE_DEV:-0}" = "1" ]; then
        EXTRA_ARGS+=("--dev")
    fi
    # Optional env sanitization between rapid invocations (default OFF)
    # Enable with: SMOKES_CLEAN_ENV=1
    local ENV_PREFIX=( )
    if [ "${SMOKES_CLEAN_ENV:-0}" = "1" ]; then
        ENV_PREFIX=(env -u NYASH_DEBUG_ENABLE -u NYASH_DEBUG_KINDS -u NYASH_DEBUG_SINK \
                        -u NYASH_RESOLVE_FIX_BRACES -u NYASH_USING_AST \
                        -u NYASH_VM_TRACE -u NYASH_VM_VERIFY_MIR -u NYASH_VM_TOLERATE_VOID \
                        -u NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN)
    fi
    # Ensure AST prelude merge is enabled for quick/integration unless explicitly disabled
    if [ -z "${NYASH_USING_AST:-}" ]; then
        export NYASH_USING_AST=1
    fi
    # -c オプションの場合は一時ファイル経由で実行
    if [ "$program" = "-c" ]; then
        local code="$1"
        shift
        local tmpfile="/tmp/nyash_test_$$.nyash"
        echo "$code" > "$tmpfile"
        # 軽量ASIFix（テスト用）: ブロック終端の余剰セミコロンを寛容に除去
        if [ "${SMOKES_ASI_STRIP_SEMI:-1}" = "1" ]; then
            sed -i -E 's/;([[:space:]]*)(\}|$)/\1\2/g' "$tmpfile" || true

        # 追加の簡易ASIFix（任意）: 同一行内のセミコロン区切りを改行へ（プラグイン最小ケースなどの互換用）
        # 影響を限定するため、明示的に SMOKES_ASI_LOOSE_SEMI=1 の時のみ有効化
        if [ "${SMOKES_ASI_LOOSE_SEMI:-0}" = "1" ]; then
            awk '{gsub(/;[[:space:]]+/, "\n"); print}' "$tmpfile" > "${tmpfile}.sm_asifix" 2>/dev/null && mv "${tmpfile}.sm_asifix" "$tmpfile" || true
        fi

        fi
        # プラグイン初期化メッセージを除外
        ensure_hako_toml
        if [ -n "${SMOKES_TIMEOUT_SEC:-}" ] && [ "${SMOKES_TIMEOUT_SEC}" != "0" ]; then
            NYASH_VM_USE_PY="$USE_PYVM" NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "${ENV_PREFIX[@]}" \
                timeout -s KILL "${SMOKES_TIMEOUT_SEC}s" "$NYASH_BIN" --backend vm "$tmpfile" "${EXTRA_ARGS[@]}" "$@" 2>&1 | filter_noise
        else
            NYASH_VM_USE_PY="$USE_PYVM" NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "${ENV_PREFIX[@]}" \
                "$NYASH_BIN" --backend vm "$tmpfile" "${EXTRA_ARGS[@]}" "$@" 2>&1 | filter_noise
        fi
        local exit_code=${PIPESTATUS[0]}
        rm -f "$tmpfile"
        return $exit_code
    else
        # 軽量ASIFix（テスト用）: ブロック終端の余剰セミコロンを寛容に除去
        if [ "${SMOKES_ASI_STRIP_SEMI:-1}" = "1" ] && [ -f "$program" ]; then
            sed -i -E 's/;([[:space:]]*)(\}|$)/\1\2/g' "$program" || true
        fi
        # プラグイン初期化メッセージを除外
        ensure_hako_toml
        if [ -n "${SMOKES_TIMEOUT_SEC:-}" ] && [ "${SMOKES_TIMEOUT_SEC}" != "0" ]; then
            NYASH_VM_USE_PY="$USE_PYVM" NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "${ENV_PREFIX[@]}" \
                timeout -s KILL "${SMOKES_TIMEOUT_SEC}s" "$NYASH_BIN" --backend vm "$program" "${EXTRA_ARGS[@]}" "$@" 2>&1 | filter_noise
        else
            NYASH_VM_USE_PY="$USE_PYVM" NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "${ENV_PREFIX[@]}" \
                "$NYASH_BIN" --backend vm "$program" "${EXTRA_ARGS[@]}" "$@" 2>&1 | filter_noise
        fi
        return ${PIPESTATUS[0]}
    fi
}

# Nyash実行ヘルパー（LLVM）
run_nyash_llvm() {
    local program="$1"
    shift
    # Allow developer to force LLVM run (env guarantees availability)
    if [ "${SMOKES_FORCE_LLVM:-0}" != "1" ]; then
        # Skip gracefully when LLVM backend is not available in this build
        # Primary check: version string advertises features (Rust/inkwell)
        if ! "$NYASH_BIN" --version 2>/dev/null | grep -q "features.*llvm"; then
            # Accept Python harness (llvmlite) as availability
            if [ -f "src/llvm_py/llvm_builder.py" ] && command -v python3 >/dev/null 2>&1 && python3 -c "import llvmlite" 2>/dev/null; then
                : # available via harness
            else
                # Fallback heuristic: binary contains LLVM harness symbols
                if ! strings "$NYASH_BIN" 2>/dev/null | grep -E -q 'ny-llvmc|NYASH_LLVM_USE_HARNESS'; then
                    log_warn "LLVM backend not available in this build; skipping LLVM run"
                    log_info "Hint: Python harness path is recommended. Ensure llvmlite is installed or build with --features llvm."
                    return 0
                fi
            fi
        fi
    fi
    # -c オプションの場合は一時ファイル経由で実行
    if [ "$program" = "-c" ]; then
        local code="$1"
        shift
        local tmpfile="/tmp/nyash_test_$$.nyash"
        echo "$code" > "$tmpfile"
        # 軽量ASIFix（テスト用）: ブロック終端の余剰セミコロンを寛容に除去
        if [ "${SMOKES_ASI_STRIP_SEMI:-1}" = "1" ]; then
            sed -i -E 's/;([[:space:]]*)(\}|$)/\1\2/g' "$tmpfile" || true

        # 追加の簡易ASIFix（任意）: 同一行内のセミコロン区切りを改行へ（プラグイン最小ケースなどの互換用）
        # 影響を限定するため、明示的に SMOKES_ASI_LOOSE_SEMI=1 の時のみ有効化
        if [ "${SMOKES_ASI_LOOSE_SEMI:-0}" = "1" ]; then
            sed -i -E 's/;[[:space:]]+/
/g' "$tmpfile" || true
        fi

        fi
        # 軽量ASIFix（テスト用）: ブロック終端の余剰セミコロンを寛容に除去
        if [ "${SMOKES_ASI_STRIP_SEMI:-1}" = "1" ] && [ -f "$program" ]; then
            sed -i -E 's/;([[:space:]]*)(\}|$)/\1\2/g' "$program" || true
        fi
        # プラグイン初期化メッセージを除外
        ensure_hako_toml
        # Harness-first policy: always use llvmlite harness unless explicitly overridden
        env PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_RUN_EMIT_EXE="${NYASH_LLVM_RUN_EMIT_EXE:-0}" NYASH_NY_LLVM_COMPILER="$NYASH_ROOT/target/release/ny-llvmc" NYASH_EMIT_EXE_NYRT="$NYASH_ROOT/target/release" NYASH_VM_USE_PY=0 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_HAKO_MIN_SEM="${NYASH_HAKO_MIN_SEM:-1}" "$NYASH_BIN" --backend llvm "$tmpfile" "$@" 2>&1 | \
            grep -v "^\[UnifiedBoxRegistry\]" | grep -v "^\[FileBox\]" | grep -v "^Net plugin:" | grep -v "^\[.*\] Plugin" | \
            grep -v '^\[using\]' | grep -v '^\[using/resolve\]' | \
            grep -v '^✅ LLVM (harness) execution completed' | grep -v '^📊 MIR Module compiled successfully' | grep -v '^📊 Functions:' | grep -v 'JSON Parse Errors:' | grep -v 'Parsing errors' | grep -v 'No parsing errors' | grep -v 'Error at line ' | \
            grep -v '^\[ny-llvmc\]' | grep -v '^\[harness\]' | grep -v '^Compiled to ' | grep -v '^/usr/bin/ld:' | grep -v '^\[deprecate\] CLI name' | grep -v '^\{"kind":"contracts_' | \
            grep -v '^\[deprecate\] \[modules\.aliases\]' | \
            grep -v '^\[warn\] dev verify:' | \
            grep -v '^🔧 Mock LLVM Backend Execution' | grep -v '^✅ Mock exit code:' | \
            grep -v '^Build with --features ' | \
            sed -E 's/^❌[[:space:]]*//' | sed -E 's/^Pipeline error: *//' | sed -E 's/\bbb[0-9]+\b/bb<ID>/g' | sed -E 's/^❌ VM fallback error: *//' | sed -E 's/^❌ Pipeline error: *//'
        local exit_code=${PIPESTATUS[0]}
        rm -f "$tmpfile"
        return $exit_code
    else
        # 軽量ASIFix（テスト用）: ブロック終端の余剰セミコロンを寛容に除去
        if [ "${SMOKES_ASI_STRIP_SEMI:-1}" = "1" ] && [ -f "$program" ]; then
            sed -i -E 's/;([[:space:]]*)(\}|$)/\1\2/g' "$program" || true
        fi
        # プラグイン初期化メッセージを除外
        ensure_hako_toml
        # Harness-first policy: always use llvmlite harness unless explicitly overridden
        env PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_RUN_EMIT_EXE="${NYASH_LLVM_RUN_EMIT_EXE:-0}" NYASH_NY_LLVM_COMPILER="$NYASH_ROOT/target/release/ny-llvmc" NYASH_EMIT_EXE_NYRT="$NYASH_ROOT/target/release" NYASH_VM_USE_PY=0 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_HAKO_MIN_SEM="${NYASH_HAKO_MIN_SEM:-1}" "$NYASH_BIN" --backend llvm "$program" "$@" 2>&1 | \
            grep -v "^\[UnifiedBoxRegistry\]" | grep -v "^\[FileBox\]" | grep -v "^Net plugin:" | grep -v "^\[.*\] Plugin" | \
            grep -v '^\[using\]' | grep -v '^\[using/resolve\]' | \
            grep -v '^✅ LLVM (harness) execution completed' | grep -v '^📊 MIR Module compiled successfully' | grep -v '^📊 Functions:' | grep -v 'JSON Parse Errors:' | grep -v 'Parsing errors' | grep -v 'No parsing errors' | grep -v 'Error at line ' | \
            grep -v '^\[ny-llvmc\]' | grep -v '^\[harness\]' | grep -v '^Compiled to ' | grep -v '^/usr/bin/ld:' | grep -v '^\[deprecate\] CLI name' | grep -v '^\{"kind":"contracts_' | \
            grep -v '^\[deprecate\] \[modules\.aliases\]' | \
            grep -v '^\[warn\] dev verify:' | \
            grep -v '^\[DEBUG-' | \
            grep -v '^🔧 Mock LLVM Backend Execution' | grep -v '^✅ Mock exit code:' | \
            grep -v '^Build with --features ' | \
            grep -v '^Result: ' | \
            sed -E 's/^❌[[:space:]]*//' | sed -E 's/^Pipeline error: *//' | sed -E 's/\bbb[0-9]+\b/bb<ID>/g' | sed -E 's/^❌ VM fallback error: *//' | sed -E 's/^❌ Pipeline error: *//'
        return ${PIPESTATUS[0]}
    fi
}

# シンプルテスト補助（スクリプト互換）
test_pass() { log_success "$1"; return 0; }
test_fail() { log_error "$1 ${2:-}"; return 1; }
test_skip() { log_warn "SKIP $1 ${2:-}"; return 0; }

# Legacy helpers (compat with older smoke snippets)
fail() {
    local msg="${1:-test failed}"
    local detail="${2:-}"
    test_fail "$msg" "$detail"
    exit 1
}

pass() {
    local msg="${1:-test passed}"
    test_pass "$msg"
    exit 0
}

# 出力比較ヘルパー
compare_outputs() {
    local expected="$1"
    local actual="$2"
    local test_name="$3"

    # Normalize outputs to absorb benign noise differences
    normalize() {
        sed -E 's/[[:space:]]+$//' | \
        sed -E 's/^❌[[:space:]]*//' | \
        sed -E 's/^Pipeline error: *//' | \
        sed -E 's/\bbb[0-9]+\b/bb<ID>/g' | \
        grep -v '^🔧 Mock LLVM Backend Execution' | \
        grep -v '^✅ Mock exit code:' | \
        grep -v 'NYASH_LLVM_USE_HARNESS' | \
        grep -v 'inkwell-legacy' | \
        grep -v '^Build with --features '
    }

    local exp_norm act_norm
    exp_norm=$(printf "%s" "$expected" | normalize)
    act_norm=$(printf "%s" "$actual" | normalize)

    if [ "$exp_norm" = "$act_norm" ]; then
        return 0
    else
        log_error "$test_name output mismatch:"
        log_error "  Expected: $exp_norm"
        log_error "  Actual:   $act_norm"
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

# LLVM 利用可否のプリフライト
# 利用可能なら0、不可ならSKIPを通知して1を返す。SMOKES_FORCE_LLVM=1で常に0。
require_llvm_or_skip() {
  if [ "${SMOKES_FORCE_LLVM:-0}" = "1" ]; then
    return 0
  fi
  if ! "$NYASH_BIN" --version 2>/dev/null | grep -q "features.*llvm"; then
    if ! command -v python3 >/dev/null 2>&1 || ! python3 -c "import llvmlite" >/dev/null 2>&1; then
      log_warn "SKIP LLVM parity Harness unavailable (install llvmlite or build with --features llvm)"
      return 1
    fi
  fi
  return 0
}

# --- 開発補助: 実行環境スタンプ（同一環境の確認用・任意） ---
_smokes_env_fingerprint() {
    (
      env | grep -E '^(NYASH|SMOKES)_' | sort
      echo "NYASH_BIN=$NYASH_BIN"
      echo "GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo nogit)"
      echo -n "BIN_SHA="; if [ -f "$NYASH_BIN" ]; then sha256sum "$NYASH_BIN" 2>/dev/null | awk '{print $1}'; else echo "none"; fi
      echo -n "PYTHON_VER="; python3 --version 2>/dev/null | awk '{print $2}' || echo "none"
      echo -n "LLVMLITE_VER="; python3 -c "import importlib, sys;
try:\n import importlib.metadata as im; print(im.version('llvmlite'))\nexcept Exception:\n print('none')" 2>/dev/null || echo "none"
    ) | sha256sum 2>/dev/null | awk '{print $1}'
}

print_env_stamp() {
    local git_hash=$(git rev-parse --short HEAD 2>/dev/null || echo nogit)
    local bin_sha="none"; if [ -f "$NYASH_BIN" ]; then bin_sha=$(sha256sum "$NYASH_BIN" 2>/dev/null | awk '{print $1}'); fi
    local py_ver=$(python3 --version 2>/dev/null | awk '{print $2}')
    local llvmlite_ver=$(python3 -c "import importlib, sys;
try:\n import importlib.metadata as im; print(im.version('llvmlite'))\nexcept Exception:\n print('none')" 2>/dev/null || true)
    local fpr=$(_smokes_env_fingerprint)
    echo "[env-stamp] git=$git_hash bin=$NYASH_BIN sha=$bin_sha" >&2
    echo "[env-stamp] python=$py_ver llvmlite=$llvmlite_ver" >&2
    echo "[env-stamp] NYASH_LLVM_USE_HARNESS=${NYASH_LLVM_USE_HARNESS:-} NYASH_LLVM_DUMP_IR=${NYASH_LLVM_DUMP_IR:-}" >&2
    echo "[env-stamp] NYASH_USING=${NYASH_USING:-} NYASH_SKIP_TOML_ENV=${NYASH_SKIP_TOML_ENV:-} SMOKES_CLEAN_ENV=${SMOKES_CLEAN_ENV:-}" >&2
    echo "[env-fpr] $fpr" >&2
}
