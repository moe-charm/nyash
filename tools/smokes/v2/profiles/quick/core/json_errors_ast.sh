#!/bin/bash
# json_errors_ast.sh - JSON error cases via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_errors_ast_$$"
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
    local bad = new ArrayBox()
    bad.push("{")
    bad.push("[1,2,,3]")
    bad.push("{\"a\" 1}")
    bad.push("{\"a\":}")

    local i = 0
    loop(i < bad.length()) {
      local s = bad.get(i)
      local parser = JsonParserModule.create_parser()
      local res = parser.parse(s)
      if parser.has_errors() or res == null {
        print("ERR")
      } else {
        print("OK")
      }
      i = i + 1
    }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
ERR
ERR
ERR
ERR
TXT
)

output=$("$NYASH_BIN" --backend vm driver.nyash 2>&1 | filter_noise)
compare_outputs "$expected" "$output" "json_errors_ast"

cd /
rm -rf "$TEST_DIR"
