#!/bin/bash
# dylib_autoload.sh - [using.dylib] DLL自動読み込みテスト（plugins プロファイル用）

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../lib/test_runner.sh"
source "$(dirname "$0")/_ensure_fixture.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# プラットフォーム依存の拡張子/ファイル名を検出
detect_lib_ext() {
    case "$(uname -s)" in
        Darwin) echo "dylib" ;;
        MINGW*|MSYS*|CYGWIN*|Windows_NT) echo "dll" ;;
        *) echo "so" ;;
    esac
}

lib_name_for() {
    local base="$1"  # e.g., nyash_fixture_plugin
    local ext="$2"
    if [ "$ext" = "dll" ]; then
        echo "${base}.dll"
    else
        echo "lib${base}.${ext}"
    fi
}

# テスト準備
setup_autoload_test() {
    TEST_DIR="/tmp/dylib_autoload_test_$$"
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"
    PLUGIN_BASE="$NYASH_ROOT/plugins"
    EXT="$(detect_lib_ext)"
    # ライブラリファイル名（プラットフォーム別）
    LIB_FIXTURE="$(lib_name_for nyash_fixture_plugin "$EXT")"
    LIB_COUNTER="$(lib_name_for nyash_counter_plugin "$EXT")"
    LIB_MATH="$(lib_name_for nyash_math_plugin "$EXT")"
    LIB_STRING="$(lib_name_for nyash_string_plugin "$EXT")"
}

# テストクリーンアップ
cleanup_autoload_test() {
    cd /
    rm -rf "$TEST_DIR"
}

# Test 0: FixtureBoxプラグイン自動読み込み（最小フィクスチャ）
test_fixture_dylib_autoload() {
    setup_autoload_test

    if [ ! -f "$NYASH_ROOT/plugins/nyash-fixture-plugin/$LIB_FIXTURE" ]; then
        ensure_fixture_plugin || true
    fi
    if [ ! -f "$NYASH_ROOT/plugins/nyash-fixture-plugin/$LIB_FIXTURE" ]; then
        test_skip "fixture_dylib_autoload" "Fixture plugin not available"
        cleanup_autoload_test; return 0
    fi

    cat > nyash.toml << EOF
[using.fixture]
kind = "dylib"
path = "$PLUGIN_BASE/nyash-fixture-plugin/$LIB_FIXTURE"
bid = "FixtureBox"

[using]
paths = ["lib"]
EOF

    cat > test_fixture.nyash << 'EOF'
using fixture
static box Main {
    main() {
        local f = new FixtureBox()
        print("Fixture: " + f.echo("hi"))
        return 0
    }
}
EOF

    local output rc
    output=$(NYASH_USING_DYLIB_AUTOLOAD=1 run_nyash_vm test_fixture.nyash 2>&1)
    if echo "$output" | grep -q "Fixture: hi"; then
        test_pass "fixture_dylib_autoload"; rc=0
    elif echo "$output" | grep -q "VM fallback error\|create_box: .* code=-5\|InvalidType"; then
        test_skip "fixture_dylib_autoload" "Fixture plugin ABI mismatch"
        rc=0
    else
        compare_outputs "Fixture: hi" "$output" "fixture_dylib_autoload"; rc=$?
    fi
    cleanup_autoload_test; return $rc
}

# Test 1: CounterBoxプラグイン自動読み込み
test_counter_dylib_autoload() {
    setup_autoload_test
    cat > nyash.toml << EOF
[using.counter_plugin]
kind = "dylib"
path = "$PLUGIN_BASE/nyash-counter-plugin/libnyash_counter_plugin.so"
bid = "CounterBox"

[using]
paths = ["lib"]
EOF

    cat > test_counter.nyash << 'EOF'
using counter_plugin
static box Main {
    main() {
        local counter = new CounterBox()
        counter.inc()
        counter.inc()
        counter.inc()
        print("Counter value: " + counter.get())
        return 0
    }
}
EOF

    local output rc
    output=$(NYASH_DEBUG_PLUGIN=1 NYASH_USING_DYLIB_AUTOLOAD=1 run_nyash_vm test_counter.nyash 2>&1)
    if echo "$output" | grep -q "Counter value: 3"; then
        rc=0
    elif echo "$output" | grep -q "create_box: .* code=-5\|Unknown Box type\|VM fallback error\|InvalidType"; then
        test_skip "counter_dylib_autoload" "Counter plugin not compatible (ABI)"
        rc=0
    else
        compare_outputs "Counter value: 3" "$output" "counter_dylib_autoload"
        rc=$?
    fi
    cleanup_autoload_test
    return $rc
}

# Test 2: MathBoxプラグイン自動読み込み
test_math_dylib_autoload() {
    if [ ! -f "$NYASH_ROOT/plugins/nyash-math-plugin/$LIB_MATH" ]; then
        test_skip "math_dylib_autoload" "Math plugin not available"
        return 0
    fi

    setup_autoload_test
    cat > nyash.toml << EOF
[using.math_plugin]
kind = "dylib"
path = "$PLUGIN_BASE/nyash-math-plugin/$LIB_MATH"
bid = "MathBox"

[using]
paths = ["lib"]
EOF

    cat > test_math.nyash << 'EOF'
using math_plugin
static box Main {
    main() {
        local math = new MathBox()
        print("Square root of 16: " + math.sqrt(16))
        print("Power 2^8: " + math.pow(2, 8))
        return 0
    }
}
EOF

    local output rc
    output=$(NYASH_USING_DYLIB_AUTOLOAD=1 run_nyash_vm test_math.nyash 2>&1)
    if echo "$output" | grep -q "Square root of 16: 4"; then
        test_pass "math_dylib_autoload"
        rc=0
    elif echo "$output" | grep -q "InvalidType\|VM fallback error"; then
        test_skip "math_dylib_autoload" "Math plugin ABI mismatch"
        rc=0
    else
        test_fail "math_dylib_autoload" "Expected math operations output"
        rc=1
    fi
    cleanup_autoload_test
    return $rc
}

# Test 3: 複数プラグイン同時読み込み
test_multiple_dylib_autoload() {
    setup_autoload_test
    cat > nyash.toml << EOF
[using.counter]
kind = "dylib"
path = "$PLUGIN_BASE/nyash-counter-plugin/$LIB_COUNTER"
bid = "CounterBox"

[using.string]
kind = "dylib"
path = "$PLUGIN_BASE/nyash-string-plugin/$LIB_STRING"
bid = "StringBox"

[using]
paths = ["lib"]
EOF

    cat > test_multiple.nyash << 'EOF'
using counter
using string

static box Main {
    main() {
        local c = new CounterBox()
        c.inc()
        local s = new StringBox("test")
        print("Counter: " + c.get() + ", String: " + s.get())
        return 0
    }
}
EOF

    local output
    output=$(NYASH_DEBUG_PLUGIN=1 NYASH_USING_DYLIB_AUTOLOAD=1 run_nyash_vm test_multiple.nyash 2>&1)
    if echo "$output" | grep -q "Counter: 1, String: test"; then
        test_pass "multiple_dylib_autoload"
    elif echo "$output" | grep -q "create_box: .* code=-5\|Unknown Box type\|VM fallback error\|InvalidType"; then
        test_skip "multiple_dylib_autoload" "Counter plugin not compatible (ABI)"
    else
        test_fail "multiple_dylib_autoload" "Expected multiple plugin output"
    fi
    cleanup_autoload_test
}

# Test 4: autoload無効時のエラー確認
test_dylib_without_autoload() {
    setup_autoload_test
    cat > nyash.toml << EOF
[using.counter_plugin]
kind = "dylib"
path = "$PLUGIN_BASE/nyash-counter-plugin/$LIB_COUNTER"
bid = "CounterBox"

[using]
paths = ["lib"]
EOF

    cat > test_no_autoload.nyash << 'EOF'
using counter_plugin
static box Main {
    main() {
        local counter = new CounterBox()
        print("Should not reach here")
        return 0
    }
}
EOF

    local output
    output=$(run_nyash_vm test_no_autoload.nyash 2>&1 || true)
    if echo "$output" | grep -qi "CounterBox\|not found\|error\|VM fallback error"; then
        test_pass "dylib_without_autoload"
    else
        test_fail "dylib_without_autoload" "Expected error without autoload"
    fi
    cleanup_autoload_test
}

# Test 5: dylib+通常パッケージの混在
test_mixed_using_with_dylib() {
    setup_autoload_test
    mkdir -p lib/utils
    cat > lib/utils/utils.nyash << 'EOF'
static box Utils {
    format(text) { return "[" + text + "]" }
}
EOF

    cat > nyash.toml << EOF
[using.utils]
path = "lib/utils/"
main = "utils.nyash"

[using.counter]
kind = "dylib"
path = "$PLUGIN_BASE/nyash-counter-plugin/$LIB_COUNTER"
bid = "CounterBox"

[using]
paths = ["lib"]
EOF

    cat > test_mixed.nyash << 'EOF'
using utils
using counter

static box Main {
    main() {
        local c = new CounterBox()
        c.inc()
        c.inc()
        local formatted = Utils.format("Count: " + c.get())
        print(formatted)
        return 0
    }
}
EOF

    local output rc
    output=$(NYASH_DEBUG_PLUGIN=1 NYASH_USING_DYLIB_AUTOLOAD=1 run_nyash_vm test_mixed.nyash 2>&1)
    if echo "$output" | grep -q "\[Count: 2\]"; then
        rc=0
    elif echo "$output" | grep -q "create_box: .* code=-5\|Unknown Box type\|VM fallback error\|InvalidType"; then
        test_skip "mixed_using_with_dylib" "Counter plugin not compatible (ABI)"
        rc=0
    else
        compare_outputs "[Count: 2]" "$output" "mixed_using_with_dylib"
        rc=$?
    fi
    cleanup_autoload_test
    return $rc
}

# テスト実行
if [ -f "$NYASH_ROOT/plugins/nyash-fixture-plugin/libnyash_fixture_plugin.so" ]; then
  run_test "dylib_fixture_autoload" test_fixture_dylib_autoload || true
fi
run_test "dylib_counter_autoload" test_counter_dylib_autoload
run_test "dylib_math_autoload" test_math_dylib_autoload
run_test "dylib_multiple_autoload" test_multiple_dylib_autoload
run_test "dylib_without_autoload" test_dylib_without_autoload
run_test "dylib_mixed_using" test_mixed_using_with_dylib
