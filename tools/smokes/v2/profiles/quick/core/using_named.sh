#!/bin/bash
# using_named.sh - [using.name]名前付きパッケージ解決テスト

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト準備
setup_using_test() {
    # テスト用一時ディレクトリ作成
    TEST_DIR="/tmp/using_named_test_$$"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"

    # nyash.toml作成
    cat > nyash.toml << 'EOF'
[using.test_package]
path = "lib/test_package/"
main = "main.nyash"

[using.aliases]
test = "test_package"

[using]
paths = ["lib"]
EOF

    # パッケージ作成
    mkdir -p lib/test_package
    cat > lib/test_package/main.nyash << 'EOF'
static box TestPackage {
    version() {
        return "1.0.0"
    }
}
EOF
}

# テストクリーンアップ
cleanup_using_test() {
    cd /
    rm -rf "$TEST_DIR"
}

# Test 1: 基本的な名前付きパッケージ解決
test_named_package_basic() {
    setup_using_test

    cat > test.nyash << 'EOF'
using test_package
static box Main {
    main() {
        print("Package version: " + TestPackage.version())
        return 0
    }
}
EOF

    local output rc
    output=$(run_nyash_vm test.nyash 2>&1)
    compare_outputs "Package version: 1.0.0" "$output" "named_package_basic"
    rc=$?
    cleanup_using_test
    return $rc
}

# Test 2: エイリアス経由の解決
test_named_package_alias() {
    setup_using_test

    cat > test_alias.nyash << 'EOF'
using test  # エイリアス使用
static box Main {
    main() {
        print("Alias resolved: " + TestPackage.version())
        return 0
    }
}
EOF

    local output rc
    output=$(run_nyash_vm test_alias.nyash 2>&1)
    compare_outputs "Alias resolved: 1.0.0" "$output" "named_package_alias"
    rc=$?
    cleanup_using_test
    return $rc
}

# Test 3: デフォルトmainエントリ
test_default_main_entry() {
    TEST_DIR="/tmp/using_default_test_$$"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"

    # mainを省略した設定
    cat > nyash.toml << 'EOF'
[using.math_utils]
path = "lib/math_utils/"
# mainは省略 → math_utils.nyashがデフォルト

[using]
paths = ["lib"]
EOF

    mkdir -p lib/math_utils
    cat > lib/math_utils/math_utils.nyash << 'EOF'
static box MathUtils {
    pi() {
        return "3.14159"
    }
}
EOF

    cat > test_default.nyash << 'EOF'
using math_utils
static box Main {
    main() {
        print("Pi value: " + MathUtils.pi())
        return 0
    }
}
EOF

    local output rc
    output=$(run_nyash_vm test_default.nyash 2>&1)
    compare_outputs "Pi value: 3.14159" "$output" "default_main_entry"
    rc=$?
    cd /
    rm -rf "$TEST_DIR"
    return $rc
}

# Test 4: 存在しないパッケージのエラー
test_missing_package_error() {
    TEST_DIR="/tmp/using_error_test_$$"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"

    cat > nyash.toml << 'EOF'
[using]
paths = ["lib"]
EOF

    cat > test_error.nyash << 'EOF'
using nonexistent_package
static box Main {
    main() {
        print("Should not reach here")
        return 0
    }
}
EOF

    local output
    output=$(run_nyash_vm test_error.nyash 2>&1 || true)

    # エラーメッセージに "not found" が含まれることを確認
    if echo "$output" | grep -q "not found\|error\|Error"; then
        test_pass "missing_package_error"
    else
        test_fail "missing_package_error" "Expected error for missing package"
    fi

    cd /
    rm -rf "$TEST_DIR"
}

# Test 5: DLL/dylib解決（オプション - プラグインが利用可能な場合のみ）
test_dylib_package() {
    # プラグインが利用可能かチェック
    if [ ! -f "$NYASH_ROOT/plugins/math/libmath.so" ]; then
        test_skip "dylib_package" "No test plugin available"
        return 0
    fi

    TEST_DIR="/tmp/using_dylib_test_$$"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"

    cat > nyash.toml << 'EOF'
[using.math_plugin]
kind = "dylib"
path = "$NYASH_ROOT/plugins/math/libmath.so"
bid = "MathBox"

[using]
paths = ["lib"]
EOF

    cat > test_dylib.nyash << 'EOF'
using math_plugin
static box Main {
    main() {
        local m = new MathBox()
        print("Dylib test: " + m.sqrt(16))
        return 0
    }
}
EOF

    local output
    output=$(run_nyash_vm test_dylib.nyash 2>&1)
    compare_outputs "Dylib test: 4" "$output" "dylib_package"

    cd /
    rm -rf "$TEST_DIR"
}

# テスト実行
run_test "using_named_basic" test_named_package_basic
run_test "using_named_alias" test_named_package_alias
run_test "using_default_main" test_default_main_entry
run_test "using_missing_error" test_missing_package_error
run_test "using_dylib_package" test_dylib_package
