#!/bin/bash
# json_error_messages_ast.sh - JSON error messages (line/column) via AST using

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=1
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_error_messages_ast_$$"
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
    local parser = JsonParserModule.create_parser()
    // Single-character object start, triggers EOF then key error at pos 2
    local res = parser.parse("{")
    if parser.has_errors() {
      local msgs = parser.get_error_messages()
      // Print first message only
      print(msgs.get(0))
    } else {
      print("NO_ERROR")
    }
    return 0
  }
}
EOF

expected='Error at line 1, column 2: Expected string key in object'
output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
compare_outputs "$expected" "$output" "json_error_messages_ast"

cd /
rm -rf "$TEST_DIR"
