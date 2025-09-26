#!/bin/bash
# using_relative_file_ast.sh - ネストした場所からの相対パス using（AST プレリュード）

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
preflight_plugins || exit 2

setup_tmp_dir() {
  TEST_DIR="/tmp/using_rel_file_$$"
  mkdir -p "$TEST_DIR"
  cd "$TEST_DIR"
}

teardown_tmp_dir() {
  cd /
  rm -rf "$TEST_DIR"
}

test_relative_alias_using_ast() {
  setup_tmp_dir

  cat > nyash.toml << 'EOF'
[using.u]
path = "lib"
main = "u.nyash"

[using]
paths = ["lib"]
EOF

  mkdir -p lib sub
  cat > lib/u.nyash << 'EOF'
static box Util { greet() { return "rel" } }
EOF

  cat > sub/main.nyash << 'EOF'
using u
static box Main {
  main() {
    print(Util.greet())
    return 0
  }
}
EOF

  export NYASH_USING_PROFILE=dev
  export NYASH_USING_AST=1
  local output rc
  output=$(run_nyash_vm sub/main.nyash 2>&1)
  if echo "$output" | grep -qx "rel"; then rc=0; else rc=1; fi
  [ $rc -eq 0 ] || { echo "$output" >&2; }
  teardown_tmp_dir
  return $rc
}

run_test "using_relative_file_ast" test_relative_alias_using_ast
