#!/bin/bash
# using_multi_prelude_dep_ast.sh - 複数プレリュード（toml パッケージ）と依存解決（ASTマージ）

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
preflight_plugins || exit 2

setup_tmp_dir() {
  TEST_DIR="/tmp/using_multi_prelude_$$"
  mkdir -p "$TEST_DIR"
  cd "$TEST_DIR"
}

teardown_tmp_dir() {
  cd /
  rm -rf "$TEST_DIR"
}

test_multi_prelude_dep_ast() {
  setup_tmp_dir

  cat > nyash.toml << 'EOF'
[using.a]
path = "lib/a"

[using.b]
path = "lib/b"

[using]
paths = ["lib"]
EOF

  mkdir -p lib/a lib/b
  cat > lib/a/a.nyash << 'EOF'
static box A { x() { return "A" } }
EOF
  cat > lib/b/b.nyash << 'EOF'
static box B { y() { return A.x() + "B" } }
EOF

  cat > main.nyash << 'EOF'
using a
using b
static box Main {
  main() {
    print(B.y())
    return 0
  }
}
EOF

  export NYASH_USING_PROFILE=prod
  export NYASH_USING_AST=1
  local output rc
  output=$(run_nyash_vm main.nyash 2>&1)
  if echo "$output" | grep -qx "AB"; then rc=0; else rc=1; fi
  [ $rc -eq 0 ] || { echo "$output" >&2; }
  teardown_tmp_dir
  return $rc
}

run_test "using_multi_prelude_dep_ast" test_multi_prelude_dep_ast
