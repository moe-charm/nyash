#!/bin/bash
# enum_macro_basic.sh - @enum macro basic tests
# Phase 19 Day 2 - @enum macro expansion test

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装

test_enum_result_ok() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r = Result.Ok(42)
    if r.is_Ok() {
      print("Ok")
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "Ok" "$output" "enum_result_ok"
}

test_enum_result_err() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r = Result.Err("failed")
    if r.is_Err() {
      print("Err")
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "Err" "$output" "enum_result_err"
}

test_enum_option_some() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Option {
  Some(value)
  None
}

static box Main {
  main() {
    local opt = Option.Some(100)
    if opt.is_Some() {
      print("Some")
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "Some" "$output" "enum_option_some"
}

test_enum_option_none() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Option {
  Some(value)
  None
}

static box Main {
  main() {
    local opt = Option.None()
    if opt.is_None() {
      print("None")
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "None" "$output" "enum_option_none"
}

test_enum_as_value() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r = Result.Ok(42)
    if r.is_Ok() {
      local v = r.as_Ok()
      print(v)
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "42" "$output" "enum_as_value"
}

# テスト実行
run_test "enum_result_ok" test_enum_result_ok
run_test "enum_result_err" test_enum_result_err
run_test "enum_option_some" test_enum_option_some
run_test "enum_option_none" test_enum_option_none
run_test "enum_as_value" test_enum_as_value
