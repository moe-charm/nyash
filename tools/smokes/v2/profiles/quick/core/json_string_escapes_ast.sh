#!/bin/bash
# json_string_escapes_ast.sh - JSON string escapes roundtrip via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_string_escapes_ast_$$"
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
    // input → expected stringify
    local inputs = new ArrayBox()
    local expect = new ArrayBox()

    inputs.push("\"A\"");            expect.push("\"A\"")
    inputs.push("\"\\n\" ");        expect.push("\"\\n\"")
    inputs.push("\"\\t\"");         expect.push("\"\\t\"")
    inputs.push("\"\\\\\"");        expect.push("\"\\\\\"")
    inputs.push("\"\\\"\"");        expect.push("\"\\\"\"")
    inputs.push("\"\\u0041\"");      expect.push("\"A\"")

    local i = 0
    loop(i < inputs.length()) {
      local s = inputs.get(i)
      local r = JsonParserModule.roundtrip_test(s)
      print(r)
      i = i + 1
    }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
"A"
"\n"
"\t"
"\\"
"\""
"A"
TXT
)

output=$("$NYASH_BIN" --backend vm driver.nyash 2>&1 | filter_noise)
compare_outputs "$expected" "$output" "json_string_escapes_ast"

cd /
rm -rf "$TEST_DIR"
