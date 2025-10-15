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

test_enum_multi_field() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Triple {
  Three(a, b, c)
}

static box Main {
  main() {
    local t = Triple.Three(10, 20, 30)
    if t.is_Three() {
      local val = t.as_Three()
      print(val)
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "10" "$output" "enum_multi_field"
}

test_enum_string_fields() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Message {
  Text(content)
  Link(url, title)
}

static box Main {
  main() {
    local msg = Message.Link("https://example.com", "Example")
    if msg.is_Link() {
      local url = msg.as_Link()
      print(url)
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "https://example.com" "$output" "enum_string_fields"
}

test_enum_tag_comparison() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Status {
  Pending(id)
  Active(id)
}

static box Main {
  main() {
    local s1 = Status.Pending(1)
    local s2 = Status.Active(2)

    if s1.is_Pending() {
      print("pending")
    }

    if s2.is_Active() {
      print("active")
    }

    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    # Check both outputs appear
    if echo "$output" | grep -q "pending" && echo "$output" | grep -q "active"; then
        echo "[PASS] enum_tag_comparison"
        return 0
    else
        echo "[FAIL] enum_tag_comparison - Expected both 'pending' and 'active', got: $output"
        return 1
    fi
}

test_enum_tostring() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Color {
  RGB(r, g, b)
  Named(name)
}

static box Main {
  main() {
    local c1 = Color.RGB(255, 0, 0)
    local str = c1.toString()
    print(str)
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    # toString() should contain "ColorBox" and the tag/fields
    if echo "$output" | grep -q "ColorBox"; then
        echo "[PASS] enum_tostring"
        return 0
    else
        echo "[FAIL] enum_tostring - Expected ColorBox in output, got: $output"
        return 1
    fi
}

test_enum_single_variant() {
    local tmpfile=$(mktemp /tmp/enum_test_XXXXXX.hako)
    cat > "$tmpfile" <<'EOF'
@enum Single {
  OnlyOne(value)
}

static box Main {
  main() {
    local s = Single.OnlyOne(42)
    if s.is_OnlyOne() {
      print("single")
    }
    return 0
  }
}
EOF
    local output
    output=$(run_nyash_vm "$tmpfile" 2>&1 | grep -v '^Result: ' | grep -v '^{' | grep -v '^\[' | grep -v 'plugin' | grep -v 'deprecate')
    rm -f "$tmpfile"
    check_exact "single" "$output" "enum_single_variant"
}

# テスト実行
run_test "enum_result_ok" test_enum_result_ok
run_test "enum_result_err" test_enum_result_err
run_test "enum_option_some" test_enum_option_some
run_test "enum_option_none" test_enum_option_none
run_test "enum_as_value" test_enum_as_value
run_test "enum_multi_field" test_enum_multi_field
run_test "enum_string_fields" test_enum_string_fields
run_test "enum_tag_comparison" test_enum_tag_comparison
run_test "enum_tostring" test_enum_tostring
run_test "enum_single_variant" test_enum_single_variant
