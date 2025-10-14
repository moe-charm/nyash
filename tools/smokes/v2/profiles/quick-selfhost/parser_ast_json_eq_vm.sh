#!/usr/bin/env bash
# parser_ast_json_eq_vm.sh — Canonical AST JSON: binary equality

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/parser_ast_json_eq_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > sample.hako << 'EOF'
function main(args) {
  print(1 == 1)
}
EOF

# BinaryOp op== canonical keys: kind, left, op, right
expected='{"kind":"Program","statements":[{"body":[{"expression":{"kind":"BinaryOp","left":{"kind":"Literal","value":{"type":"int","value":1}},"op":"==","right":{"kind":"Literal","value":{"type":"int","value":1}}},"kind":"Print"}],"kind":"FunctionDeclaration","name":"main","override":false,"params":["args"],"static":false}]}'

output=$("$NYASH_BIN" --dump-ast-json sample.hako 2>&1 | filter_noise | tr -d '\n')
if compare_outputs "$expected" "$output" "parser_ast_json_eq_vm"; then
  test_pass parser_ast_json_eq_vm
else
  test_fail parser_ast_json_eq_vm
fi

cd /
rm -rf "$TEST_DIR"

