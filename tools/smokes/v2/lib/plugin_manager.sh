#!/bin/bash
# plugin_manager.sh - プラグイン設定管理
# Rust VM (動的) vs LLVM (静的) プラグイン設定の統一管理

# set -eは使わない（個々のテストが失敗しても全体を続行するため）
set -uo pipefail

# プラグイン設定検出
detect_plugin_mode() {
    # 環境変数で明示的指定があれば優先
    if [ "${SMOKES_PLUGIN_MODE:-}" = "dynamic" ]; then
        echo "dynamic"
        return 0
    elif [ "${SMOKES_PLUGIN_MODE:-}" = "static" ]; then
        echo "static"
        return 0
    fi

    # バックエンド検出による自動判定
    if [ "${NYASH_BACKEND:-}" = "llvm" ]; then
        echo "static"
    else
        echo "dynamic"
    fi
}

# 動的プラグイン整合性チェック
check_dynamic_plugins() {
    local plugin_dir="plugins"
    local required_plugins=("stringbox" "integerbox" "mathbox")
    local missing_plugins=()

    if [ ! -d "$plugin_dir" ]; then
        echo "[WARN] Plugin directory not found: $plugin_dir" >&2
        return 0  # 警告のみ、エラーにしない
    fi

    for plugin in "${required_plugins[@]}"; do
        if [ ! -f "$plugin_dir/${plugin}/${plugin}.so" ]; then
            missing_plugins+=("$plugin")
        fi
    done

    if [ ${#missing_plugins[@]} -ne 0 ]; then
        echo "[WARN] Missing dynamic plugins: ${missing_plugins[*]}" >&2
        echo "[INFO] Run: tools/plugin-tester/target/release/plugin-tester build-all" >&2
        # Export skip hint for test runner when requested
        if [ "${SMOKES_SKIP_WHEN_PLUGINS_MISSING:-0}" = "1" ]; then
            export SMOKES_SKIP_CUR_TEST=1
            export SMOKES_SKIP_REASON="plugins missing: ${missing_plugins[*]}"
        fi
        return 0  # 警告のみ（skipはランナー側で処理）
    fi

    echo "[INFO] Dynamic plugins check passed" >&2
    return 0
}

# 静的プラグイン整合性チェック
check_static_plugins() {
    # LLVM対応のプラグインがビルドに含まれているかチェック
    if ! ./target/release/nyash --version 2>/dev/null | grep -q "features.*llvm"; then
        echo "[WARN] LLVM backend not available in current build" >&2
        echo "[INFO] Static plugin tests may fail" >&2
        return 0  # 警告のみ
    fi

    echo "[INFO] Static plugins check passed" >&2
    return 0
}

# プラグイン整合性チェック（統合）
check_plugin_integrity() {
    local plugin_mode
    plugin_mode=$(detect_plugin_mode)

    echo "[INFO] Plugin mode: $plugin_mode" >&2

    case "$plugin_mode" in
        "dynamic")
            check_dynamic_plugins
            ;;
        "static")
            check_static_plugins
            ;;
        *)
            echo "[ERROR] Unknown plugin mode: $plugin_mode" >&2
            return 1
            ;;
    esac
}

# プラグイン環境設定
setup_plugin_env() {
    local plugin_mode
    plugin_mode=$(detect_plugin_mode)

    case "$plugin_mode" in
        "dynamic")
            # 動的プラグイン用環境設定
            export NYASH_DISABLE_PLUGINS=0
            unset NYASH_BACKEND  # デフォルトVM使用
            echo "[INFO] Configured for dynamic plugins (Rust VM)" >&2
            ;;
        "static")
            # 静的プラグイン用環境設定
            export NYASH_DISABLE_PLUGINS=1  # コアプラグインのみ
            export NYASH_BACKEND=llvm
            echo "[INFO] Configured for static plugins (LLVM)" >&2
            ;;
    esac
}

# 推奨プラグイン設定表示
show_plugin_recommendations() {
    cat << 'EOF' >&2
===============================================
Plugin Configuration Recommendations
===============================================

For Development (Fast Iteration):
  SMOKES_PLUGIN_MODE=dynamic
  → Uses Rust VM with .so plugin loading
  → Fast build, good for debugging

For CI/Production (Stable):
  SMOKES_PLUGIN_MODE=static
  → Uses LLVM with compiled-in plugins
  → Slower build, more reliable

Auto-detection:
  - Default: dynamic (Rust VM)
  - NYASH_BACKEND=llvm: static
  - Override with SMOKES_PLUGIN_MODE

===============================================
EOF
}

# プラグインテスター統合
rebuild_plugins() {
    local plugin_tester="tools/plugin-tester/target/release/plugin-tester"

    if [ ! -f "$plugin_tester" ]; then
        echo "[WARN] Plugin tester not found: $plugin_tester" >&2
        echo "[INFO] Run: cd tools/plugin-tester && cargo build --release" >&2
        return 1
    fi

    echo "[INFO] Rebuilding all plugins..." >&2
    if "$plugin_tester" build-all; then
        echo "[INFO] Plugin rebuild completed" >&2
        return 0
    else
        echo "[ERROR] Plugin rebuild failed" >&2
        return 1
    fi
}

# 使用例とヘルプ
show_help() {
    cat << 'EOF'
Plugin Manager for Smoke Tests v2

Usage:
  source lib/plugin_manager.sh

Functions:
  detect_plugin_mode           - Auto-detect plugin mode
  check_plugin_integrity       - Verify plugin setup
  setup_plugin_env            - Configure environment
  show_plugin_recommendations  - Show config guidance
  rebuild_plugins             - Rebuild all plugins

Environment Variables:
  SMOKES_PLUGIN_MODE=dynamic|static  - Force plugin mode
  NYASH_BACKEND=llvm                 - Auto-selects static

Examples:
  # Force dynamic mode
  SMOKES_PLUGIN_MODE=dynamic ./run.sh --profile quick

  # Force static mode
  SMOKES_PLUGIN_MODE=static ./run.sh --profile integration
EOF
}
