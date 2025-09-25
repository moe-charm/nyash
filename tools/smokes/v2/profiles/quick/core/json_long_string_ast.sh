#!/bin/bash
# json_long_string_ast.sh - Long JSON string roundtrip via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=1
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_long_string_ast_$$"
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
    // Long ASCII string with common escapes; repeated to increase size
    local payload = "A long string with quote \" and backslash \\ and newline \\n and tab \\t. "
    local s = "\"" + payload + payload + payload + payload + "\""  // JSON string literal
    local out = JsonParserModule.roundtrip_test(s)
    print(out)
    return 0
  }
}
EOF

# Expected is the exact JSON string used (payload repeated 4x inside quotes)
expected=$(cat << 'TXT'
"A long string with quote \" and backslash \\ and newline \n and tab \t. A long string with quote \" and backslash \\ and newline \n and tab \t. A long string with quote \" and backslash \\ and newline \n and tab \t. A long string with quote \" and backslash \\ and newline \n and tab \t. "
TXT
)

output=$(run_nyash_vm driver.nyash)
compare_outputs "$expected" "$output" "json_long_string_ast"

cd /
rm -rf "$TEST_DIR"
