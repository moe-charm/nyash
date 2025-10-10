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
    # Logical→actual artifact mapping (any candidate satisfies the requirement)
    declare -A candidates
    candidates[stringbox]="nyash-string-plugin/libnyash_string_plugin.so nyash-string-plugin/libnyash_string_plugin.a"
    candidates[integerbox]="nyash-integer-plugin/libnyash_integer_plugin.so nyash-integer-plugin/libnyash_integer_plugin.a"
    candidates[mathbox]="nyash-math-plugin/libnyash_math_plugin.so nyash-math-plugin/libnyash_math_plugin.a nyash-math/libnyash_math.so"
    candidates[arraybox]="nyash-array-plugin/libnyash_array_plugin.so nyash-array-plugin/libnyash_array_plugin.a"
    candidates[mapbox]="nyash-map-plugin/libnyash_map_plugin.so nyash-map-plugin/libnyash_map_plugin.a"
    candidates[filebox]="nyash-filebox-plugin/libnyash_filebox_plugin.so nyash-filebox-plugin/libnyash_filebox_plugin.a"

    local required_plugins=("stringbox" "integerbox" "mathbox" "arraybox" "mapbox" "filebox")
    local missing_plugins=()

    # Always rebuild when auto-build is enabled to refresh .so artifacts
    if [ "${SMOKES_AUTO_BUILD_PLUGINS:-1}" = "1" ]; then
        rebuild_plugins || true
    fi


    if [ ! -d "$plugin_dir" ]; then
        echo "[WARN] Plugin directory not found: $plugin_dir" >&2
        return 0  # 警告のみ、エラーにしない
    fi

    for req in "${required_plugins[@]}"; do
        local ok=0
        for cand in ${candidates[$req]:-}; do
            if [ -f "$plugin_dir/$cand" ]; then ok=1; break; fi
        done
        if [ $ok -eq 0 ]; then missing_plugins+=("$req"); fi
    done

    if [ ${#missing_plugins[@]} -ne 0 ]; then
        if [ "${SMOKES_AUTO_BUILD_PLUGINS:-1}" = "1" ]; then
            rebuild_plugins || true
            # Recheck
            local still_missing=()
            for req in "${required_plugins[@]}"; do
                local ok=0
                for cand in ${candidates[$req]:-}; do
                    if [ -f "$plugin_dir/$cand" ]; then ok=1; break; fi
                done
                if [ $ok -eq 0 ]; then still_missing+=("$req"); fi
            done
            if [ ${#still_missing[@]} -eq 0 ]; then
                echo "[INFO] Dynamic plugins present after rebuild" >&2
                return 0
            fi
            missing_plugins=("${still_missing[@]}")
        fi
        echo "[WARN] Missing dynamic plugins: ${missing_plugins[*]}" >&2
        echo "[INFO] Run: tools/plugin-tester/target/release/plugin-tester build-all" >&2
        if [ "${SMOKES_SKIP_WHEN_PLUGINS_MISSING:-0}" = "1" ]; then
            export SMOKES_SKIP_CUR_TEST=1
            export SMOKES_SKIP_REASON="plugins missing: ${missing_plugins[*]}"
        fi
        return 0
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
            # 明示的に nyash.toml を指定して v2 ホストにプロバイダ登録させる
            if [ -z "${NYASH_PLUGIN_CONFIG:-}" ]; then
              if [ -f "$NYASH_ROOT/nyash.toml" ]; then
                export NYASH_PLUGIN_CONFIG="$NYASH_ROOT/nyash.toml"
              elif [ -f "nyash.toml" ]; then
                export NYASH_PLUGIN_CONFIG="nyash.toml"
              fi
            fi
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

    echo "[INFO] Rebuilding plugins (fallback-safe)..." >&2
    if [ -f "$plugin_tester" ]; then
        # Prefer tester when it supports build-all; otherwise fall back to cargo
        if "$plugin_tester" help 2>/dev/null | grep -q "build-all"; then
            if "$plugin_tester" build-all; then
                echo "[INFO] Plugin rebuild completed (tester)" >&2
                return 0
            fi
            echo "[WARN] Plugin tester build-all failed; falling back to cargo" >&2
        fi
    else
        echo "[WARN] Plugin tester not found: $plugin_tester" >&2
        echo "[INFO] Building via cargo fallback" >&2
    fi

    # Minimal required set for plugin-on smokes
    if cargo build --release -p nyash-array-plugin -p nyash-map-plugin -p nyash-string-plugin -p nyash-filebox-plugin >/dev/null 2>&1; then
        if [ -f "plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin.so" ]; then
            cp -f "plugins/nyash-filebox-plugin/target/release/libnyash_filebox_plugin.so" \
                  "plugins/nyash-filebox-plugin/libnyash_filebox_plugin.so" 2>/dev/null || true
        fi
        echo "[INFO] Plugin rebuild completed (cargo)" >&2
        return 0
    fi
    echo "[ERROR] Plugin rebuild failed (cargo)" >&2
    return 1
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