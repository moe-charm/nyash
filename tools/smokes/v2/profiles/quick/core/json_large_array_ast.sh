#!/bin/bash
# json_large_array_ast.sh - Large array roundtrip via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=1
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_large_array_ast_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > nyash.toml << EOF
[using.json_native]
path = "$NYASH_ROOT/apps/lib/json_native/"
main = "parser/parser.nyash"

[using.aliases]
json = "json_native"
EOF

cat > driver.nyash << 'EOF'
using json_native as JsonParserModule

static box Main {
  main() {
    local arr = "[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19]"
    local out = JsonParserUtils.roundtrip_test(arr)
    print(out)
    return 0
  }
}
EOF

expected='[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19]'
output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
compare_outputs "$expected" "$output" "json_large_array_ast"

cd /
rm -rf "$TEST_DIR"
