#!/bin/bash
# json_deep_nesting_ast.sh - Deeply nested arrays/objects roundtrip via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=1
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_deep_nesting_ast_$$"
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
    // Build a 20-level nested array: [[[[...0...]]]]
    local open = "[[[[[[[[[[[[[[[[[[["  // 20 '['
    local close = "]]]]]]]]]]]]]]]]]]]"  // 20 ']'
    local arr = open + "0" + close

    // Build a 10-level nested object: {"a":{"a":{..."a":0...}}}
    local i = 0
    local obj = "0"
    loop(i < 10) {
      obj = "{\\"a\\":" + obj + "}"
      i = i + 1
    }

    local samples = new ArrayBox()
    samples.push(arr)
    samples.push(obj)

    local k = 0
    loop(k < samples.length()) {
      local s = samples.get(k)
      local r = JsonParserModule.roundtrip_test(s)
      print(r)
      k = k + 1
    }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
[[[[[[[[[[[[[[[[[[[0]]]]]]]]]]]]]]]]]]
{"a":{"a":{"a":{"a":{"a":{"a":{"a":{"a":{"a":{"a":0}}}}}}}}}}
TXT
)

output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
compare_outputs "$expected" "$output" "json_deep_nesting_ast"

cd /
rm -rf "$TEST_DIR"
