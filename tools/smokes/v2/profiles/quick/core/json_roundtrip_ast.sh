#!/bin/bash
# json_roundtrip_ast.sh - JSON parse/stringify roundtrip via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_roundtrip_ast_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
using "apps/lib/json_native/parser/parser.nyash" as JsonParserModule

static box Main {
  main() {
    local samples = new ArrayBox()
    samples.push("null")
    samples.push("true")
    samples.push("false")
    samples.push("42")
    samples.push("\"hello\"")
    samples.push("[]")
    samples.push("{}")
    samples.push("{\"a\":1}")

    local i = 0
    loop(i < samples.length()) {
      local s = samples.get(i)
      local r = JsonParserModule.roundtrip_test(s)
      // Print each roundtrip result on its own line
      print(r)
      i = i + 1
    }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
null
true
false
42
"hello"
[]
{}
{"a":1}
TXT
)

output=$(run_nyash_vm driver.nyash)
compare_outputs "$expected" "$output" "json_roundtrip_ast"

cd /
rm -rf "$TEST_DIR"
