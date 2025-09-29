#!/bin/bash
# preflight.sh - 前処理・環境チェック
# テスト実行前の統一前処理システム

# set -eは使わない（個々のテストが失敗しても全体を続行するため）
set -uo pipefail

# プリフライトチェック実行
preflight_all() {
    echo "[INFO] Starting preflight checks..." >&2

    # 基本環境チェック
    if ! preflight_basic_env; then
        echo "[ERROR] Basic environment check failed" >&2
        return 1
    fi

    # Nyashビルド確認
    if ! preflight_nyash_build; then
        echo "[ERROR] Nyash build check failed" >&2
        return 1
    fi

    # プラグイン整合性チェック
    if ! preflight_plugins; then
        echo "[ERROR] Plugin integrity check failed" >&2
        return 1
    fi

    # 依存関係チェック
    if ! preflight_dependencies; then
        echo "[ERROR] Dependency check failed" >&2
        return 1
    fi

    echo "[INFO] All preflight checks passed ✓" >&2
    return 0
}

# 基本環境チェック
preflight_basic_env() {
    local required_commands=("cargo" "grep" "jq" "bc" "timeout")
    local missing_commands=()

    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_commands+=("$cmd")
        fi
    done

    if [ ${#missing_commands[@]} -ne 0 ]; then
        echo "[ERROR] Missing required commands: ${missing_commands[*]}" >&2
        echo "[INFO] Please install missing commands and try again" >&2
        return 1
    fi

    # ディスク容量チェック（最低1GB）
    local available_mb
    available_mb=$(df . | tail -n1 | awk '{print int($4/1024)}')
    if [ "$available_mb" -lt 1024 ]; then
        echo "[WARN] Low disk space: ${available_mb}MB available" >&2
        echo "[INFO] Recommend at least 1GB for safe operation" >&2
    fi

    echo "[INFO] Basic environment: OK" >&2
    return 0
}

# Nyashビルド確認
preflight_nyash_build() {
    local nyash_exe="./target/release/nyash"

    # バイナリ存在確認
    if [ ! -f "$nyash_exe" ]; then
        echo "[ERROR] Nyash executable not found: $nyash_exe" >&2
        echo "[INFO] Run 'cargo build --release' to build Nyash" >&2
        return 1
    fi

    # バイナリ実行可能性確認
    if [ ! -x "$nyash_exe" ]; then
        echo "[ERROR] Nyash executable is not executable: $nyash_exe" >&2
        chmod +x "$nyash_exe" 2>/dev/null || true
        if [ ! -x "$nyash_exe" ]; then
            echo "[ERROR] Failed to make executable" >&2
            return 1
        fi
        echo "[INFO] Made Nyash executable" >&2
    fi

    # 基本動作確認
    if ! "$nyash_exe" --version >/dev/null 2>&1; then
        echo "[ERROR] Nyash version check failed" >&2
        echo "[INFO] Binary may be corrupted, try rebuilding" >&2
        return 1
    fi

    # バックエンド対応状況確認
    local version_output
    version_output=$("$nyash_exe" --version 2>&1)

    if echo "$version_output" | grep -q "features.*llvm"; then
        echo "[INFO] LLVM backend (Rust/inkwell): Available" >&2
    else
        # If Python harness (llvmlite) is present, treat LLVM as available via harness
        if [ -f "src/llvm_py/llvm_builder.py" ] && command -v python3 >/dev/null 2>&1 && python3 -c "import llvmlite" 2>/dev/null; then
            echo "[INFO] LLVM backend: Available via Python llvmlite harness" >&2
        else
            echo "[WARN] LLVM backend: Not available in this build" >&2
        fi
    fi

    if echo "$version_output" | grep -q "features.*cranelift"; then
        echo "[INFO] Cranelift JIT: Available" >&2
    else
        echo "[WARN] Cranelift JIT: Not available in this build" >&2
    fi

    echo "[INFO] Nyash build: OK" >&2
    return 0
}

# プラグイン整合性チェック
preflight_plugins() {
    # plugin_manager.shから関数をインポート
    local plugin_manager_path
    plugin_manager_path="$(dirname "${BASH_SOURCE[0]}")/plugin_manager.sh"

    if [ ! -f "$plugin_manager_path" ]; then
        echo "[ERROR] Plugin manager not found: $plugin_manager_path" >&2
        return 1
    fi

    source "$plugin_manager_path"

    # プラグイン整合性チェック実行
    if ! check_plugin_integrity; then
        echo "[ERROR] Plugin integrity check failed" >&2
        echo "[INFO] Try rebuilding plugins with: tools/plugin-tester/target/release/plugin-tester build-all" >&2
        return 1
    fi

    # Provider Verify（段階導入）: nyash.toml の [verify.required_methods] / [types.*.required_methods]
    # 既定 warn。SMOKES_PROVIDER_VERIFY_MODE=strict でエラー化。
    local verify_mode="${SMOKES_PROVIDER_VERIFY_MODE:-warn}"
    if [ -f "./target/release/nyash" ]; then
        local tmp_preflight
        tmp_preflight="/tmp/nyash_preflight_empty_$$.ny"
        echo "/* preflight */" > "$tmp_preflight"
        if NYASH_PROVIDER_VERIFY="$verify_mode" ./target/release/nyash "$tmp_preflight" >/dev/null 2>&1; then
            echo "[INFO] Provider verify ($verify_mode): OK" >&2
        else
            if [ "$verify_mode" = "strict" ]; then
                echo "[ERROR] Provider verify failed (strict)" >&2
                rm -f "$tmp_preflight" 2>/dev/null || true
                return 1
            else
                echo "[WARN] Provider verify reported issues (mode=$verify_mode)" >&2
            fi
        fi
        rm -f "$tmp_preflight" 2>/dev/null || true
    fi

    echo "[INFO] Plugin integrity: OK" >&2
    return 0
}

# 依存関係チェック
preflight_dependencies() {
    # Python LLVM関連チェック（オプション）
    if [ -f "src/llvm_py/llvm_builder.py" ]; then
        if command -v python3 &> /dev/null; then
            if python3 -c "import llvmlite" 2>/dev/null; then
                echo "[INFO] Python LLVM: Available" >&2
            else
                echo "[WARN] Python LLVM: llvmlite not installed" >&2
                echo "[INFO] Install with: pip install llvmlite" >&2
            fi
        else
            echo "[WARN] Python LLVM: python3 not available" >&2
        fi
    fi

    # プラグインテスター確認（オプション）
    local plugin_tester="tools/plugin-tester/target/release/plugin-tester"
    if [ -f "$plugin_tester" ]; then
        echo "[INFO] Plugin tester: Available" >&2
    else
        echo "[WARN] Plugin tester: Not built" >&2
        echo "[INFO] Build with: cd tools/plugin-tester && cargo build --release" >&2
    fi

    # Git確認（オプション）
    if command -v git &> /dev/null && [ -d ".git" ]; then
        local git_status
        if git_status=$(git status --porcelain 2>/dev/null) && [ -n "$git_status" ]; then
            echo "[WARN] Git: Working directory has uncommitted changes" >&2
            echo "[INFO] Consider committing changes before running tests" >&2
        else
            echo "[INFO] Git: Working directory clean" >&2
        fi
    fi

    echo "[INFO] Dependencies: OK" >&2
    return 0
}

# 環境情報出力
show_environment_info() {
    cat << 'EOF'
===============================================
Environment Information
===============================================
EOF

    echo "System: $(uname -a)"
    echo "Working Directory: $(pwd)"
    echo "Date: $(date)"
    echo ""

    # Cargo情報
    if command -v cargo &> /dev/null; then
        echo "Cargo: $(cargo --version)"
    fi

    # Rust情報
    if command -v rustc &> /dev/null; then
        echo "Rust: $(rustc --version)"
    fi

    # Python情報
    if command -v python3 &> /dev/null; then
        echo "Python: $(python3 --version)"
    fi

    echo ""

    # Nyash情報
    if [ -f "./target/release/nyash" ]; then
        echo "Nyash: $(./target/release/nyash --version 2>&1 | head -n1)"
        echo "Features: $(./target/release/nyash --version 2>&1 | grep features || echo 'default')"
    fi

    echo ""
    echo "==============================================="
}

# プリフライト修復
preflight_repair() {
    echo "[INFO] Attempting automatic repairs..." >&2

    # Nyashバイナリの実行権限修復
    if [ -f "./target/release/nyash" ] && [ ! -x "./target/release/nyash" ]; then
        chmod +x "./target/release/nyash" 2>/dev/null || true
        echo "[INFO] Fixed Nyash executable permissions" >&2
    fi

    # プラグイン再ビルド（オプション）
    if [ "${PREFLIGHT_REBUILD_PLUGINS:-0}" = "1" ]; then
        local plugin_tester="tools/plugin-tester/target/release/plugin-tester"
        if [ -f "$plugin_tester" ]; then
            echo "[INFO] Rebuilding plugins..." >&2
            if "$plugin_tester" build-all 2>/dev/null; then
                echo "[INFO] Plugin rebuild completed" >&2
            else
                echo "[WARN] Plugin rebuild failed" >&2
            fi
        fi
    fi

    echo "[INFO] Repair attempts completed" >&2
}

# 使用例とヘルプ
show_preflight_help() {
    cat << 'EOF'
Preflight Checker for Smoke Tests v2

Usage:
  source lib/preflight.sh

Functions:
  preflight_all              - Run all preflight checks
  preflight_basic_env        - Check basic environment
  preflight_nyash_build      - Check Nyash build
  preflight_plugins          - Check plugin integrity
  preflight_dependencies     - Check optional dependencies
  show_environment_info      - Display environment info
  preflight_repair          - Attempt automatic repairs

Environment Variables:
  PREFLIGHT_REBUILD_PLUGINS=1  - Auto-rebuild plugins during repair

Examples:
  # Full preflight check
  source lib/preflight.sh && preflight_all

  # Show environment info
  source lib/preflight.sh && show_environment_info

  # Repair with plugin rebuild
  PREFLIGHT_REBUILD_PLUGINS=1 source lib/preflight.sh && preflight_repair
EOF
}
