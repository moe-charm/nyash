#!/bin/bash
# parser_ast_json_canonical_vm.sh - Verify canonical AST JSON dump is stable

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/parser_ast_json_canonical_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > sample.hako << 'EOF'
function main(args) {
  print(1 + 2)
}
EOF

# Expected canonical JSON (object keys sorted lexicographically within each object)
expected='{"kind":"Program","statements":[{"body":[{"expression":{"kind":"BinaryOp","left":{"kind":"Literal","value":{"type":"int","value":1}},"op":"+","right":{"kind":"Literal","value":{"type":"int","value":2}}},"kind":"Print"}],"kind":"FunctionDeclaration","name":"main","override":false,"params":["args"],"static":false}]}'

output=$("$NYASH_BIN" --dump-ast-json sample.hako 2>&1 | filter_noise | tr -d '\n' )

if compare_outputs "$expected" "$output" "parser_ast_json_canonical_vm"; then
  test_pass parser_ast_json_canonical_vm
else
  test_fail parser_ast_json_canonical_vm
fi

cd /
rm -rf "$TEST_DIR"
