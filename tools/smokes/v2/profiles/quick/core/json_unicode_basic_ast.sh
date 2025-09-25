#!/bin/bash
# json_unicode_basic_ast.sh - Basic \uXXXX handling via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=1
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_unicode_basic_ast_$$"
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
    // Case 1: ASCII via \uXXXX becomes plain letters after roundtrip
    local s1 = "\"A\\u0041\\u005A\""  // => "AAZ"
    print(JsonParserUtils.roundtrip_test(s1))

    // Case 2: control via \u000A becomes newline, stringifier emits \n
    local s2 = "\"line\\u000Aend\""
    print(JsonParserUtils.roundtrip_test(s2))

    return 0
  }
}
EOF

expected=$(cat << 'TXT'
"AAZ"
"line\nend"
TXT
)

output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
compare_outputs "$expected" "$output" "json_unicode_basic_ast"

cd /
rm -rf "$TEST_DIR"
