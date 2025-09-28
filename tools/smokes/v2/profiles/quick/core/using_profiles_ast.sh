#!/bin/bash
# using_profiles_ast.sh - using プロファイル（dev/prod）× AST プレリュードの基本動作チェック

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
preflight_plugins || exit 2

# Quick policy: AST prelude merge is experimental; cover in integration/full
test_skip "using_profiles_ast (quick)" "AST prelude merge experimental; run in integration/full" && exit 0

setup_tmp_dir() {
  TEST_DIR="/tmp/using_profiles_ast_$$"
  mkdir -p "$TEST_DIR"
  cd "$TEST_DIR"
}

teardown_tmp_dir() {
  cd /
  rm -rf "$TEST_DIR"
}

# Test A: dev プロファイルでも `using "file"` は禁止（SSOT 徹底）
test_dev_file_using_forbidden_ast() {
  setup_tmp_dir

  # nyash.toml（paths だけで十分）
  cat > nyash.toml << 'EOF'
[using]
paths = ["lib"]
EOF

  mkdir -p lib
  cat > lib/u.nyash << 'EOF'
static box Util {
  greet() { return "hi" }
}
EOF

  cat > main.nyash << 'EOF'
using "lib/u.nyash"
static box Main {
  main() {
    print(Util.greet())
    return 0
  }
}
EOF

  local output
  # dev + AST モード（失敗が正）
  export NYASH_USING_PROFILE=dev
  export NYASH_USING_AST=1
  output=$(run_nyash_vm main.nyash 2>&1 || true)
  if echo "$output" | grep -qi "disallowed\|nyash.toml \[using\]"; then
    test_pass "dev_file_using_forbidden_ast"
  else
    test_fail "dev_file_using_forbidden_ast" "expected guidance error, got: $output"
  fi
  teardown_tmp_dir
  return 0
}

# Test B: prod プロファイルでは `using "file"` は拒否（ガイダンス付きエラー）
test_prod_file_using_forbidden_ast() {
  setup_tmp_dir

  cat > nyash.toml << 'EOF'
[using]
paths = ["lib"]
EOF

  mkdir -p lib
  cat > lib/u.nyash << 'EOF'
static box Util { greet() { return "x" } }
EOF

  cat > main.nyash << 'EOF'
using "lib/u.nyash"
static box Main { main() { print(Util.greet()); return 0 } }
EOF

  local output
  # prod + AST モード（失敗が正）
  export NYASH_USING_PROFILE=prod
  export NYASH_USING_AST=1
  output=$(run_nyash_vm main.nyash 2>&1 || true)
  if echo "$output" | grep -qi "disallowed\|nyash.toml \[using\]"; then
    test_pass "prod_file_using_forbidden_ast"
  else
    test_fail "prod_file_using_forbidden_ast" "expected guidance error, got: $output"
  fi

  teardown_tmp_dir
}

# Test C: prod プロファイルで alias/package は許可（AST プレリュードで読み込み）
test_prod_alias_package_ok_ast() {
  setup_tmp_dir

  cat > nyash.toml << 'EOF'
[using.u]
path = "lib/u"

[using]
paths = ["lib"]
EOF

  mkdir -p lib/u
  # main 省略 → leaf 名と同名の .nyash がデフォルト（u/u.nyash）
  cat > lib/u/u.nyash << 'EOF'
static box Util {
  version() { return "ok" }
}
EOF

  cat > main.nyash << 'EOF'
using u
static box Main {
  main() {
    print(Util.version())
    return 0
  }
}
EOF

  local output rc
  export NYASH_USING_PROFILE=prod
  export NYASH_USING_AST=1
  output=$(run_nyash_vm main.nyash 2>&1)
  if echo "$output" | grep -qx "ok"; then rc=0; else rc=1; fi
  [ $rc -eq 0 ] || { echo "$output" >&2; }
  teardown_tmp_dir
  return $rc
}

run_test "using_dev_file_forbidden_ast" test_dev_file_using_forbidden_ast
run_test "using_prod_file_forbidden_ast" test_prod_file_using_forbidden_ast
run_test "using_prod_alias_ok_ast" test_prod_alias_package_ok_ast
