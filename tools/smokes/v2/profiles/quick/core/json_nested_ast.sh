#!/bin/bash
# json_nested_ast.sh - Nested arrays/objects roundtrip via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=1
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_nested_ast_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Ensure LLVM harness script is discoverable from CWD
mkdir -p tools
cp -f "$NYASH_ROOT/tools/llvmlite_harness.py" tools/ 2>/dev/null || true

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
    local samples = new ArrayBox()
    samples.push("[1,[2,3],{\"x\":[4]}]")
    samples.push("{\"a\":{\"b\":[1,2]},\"c\":\"d\"}")
    samples.push("{\"n\":-1e-3,\"z\":0.0}")

    local i = 0
    loop(i < samples.length()) {
      local s = samples.get(i)
      local r = JsonParserUtils.roundtrip_test(s)
      print(r)
      i = i + 1
    }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
[1,[2,3],{"x":[4]}]
{"a":{"b":[1,2]},"c":"d"}
{"n":-1e-3,"z":0.0}
TXT
)

output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
compare_outputs "$expected" "$output" "json_nested_ast"

cd /
rm -rf "$TEST_DIR"
