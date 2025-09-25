#!/bin/bash
# json_error_positions_ast.sh - Error UX (line/column) checks via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_error_positions_ast_$$"
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
    // Case 1: missing comma in array
    local p1 = JsonParserModule.create_parser()
    local r1 = p1.parse("[1 2]")
    if p1.has_errors() { print(p1.get_error_messages().get(0)) } else { print("NO_ERROR") }

    // Case 2: unexpected character in literal
    local p2 = JsonParserModule.create_parser()
    local r2 = p2.parse("truX")
    if p2.has_errors() { print(p2.get_error_messages().get(0)) } else { print("NO_ERROR") }

    // Case 3: object missing comma (multi-line)
    local json3 = "{\n\"a\": 1\n  2\n}"
    local p3 = JsonParserModule.create_parser()
    local r3 = p3.parse(json3)
    if p3.has_errors() { print(p3.get_error_messages().get(0)) } else { print("NO_ERROR") }

    return 0
  }
}
EOF

expected=$(cat << 'TXT'
Error at line 1, column 4: Expected ',' or ']' in array
Error at line 1, column 1: Lexer error: Unknown keyword: truX
Error at line 3, column 3: Expected ',' or '}' in object
TXT
)

output=$("$NYASH_BIN" --backend vm driver.nyash 2>&1 | filter_noise)
compare_outputs "$expected" "$output" "json_error_positions_ast"

cd /
rm -rf "$TEST_DIR"
