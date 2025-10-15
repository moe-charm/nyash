#!/usr/bin/env bash
# parser_ast_json_if_else_vm.sh — Canonical AST JSON: if/else shape

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/parser_ast_json_ifelse_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > sample.hako << 'EOF'
function main(args) {
  if (1) { print(1) } else { print(2) }
}
EOF

# If canonical keys sorted lexicographically: condition, else, kind, then
expected='{"kind":"Program","statements":[{"body":[{"condition":{"kind":"Literal","value":{"type":"int","value":1}},"else":[{"expression":{"kind":"Literal","value":{"type":"int","value":2}},"kind":"Print"}],"kind":"If","then":[{"expression":{"kind":"Literal","value":{"type":"int","value":1}},"kind":"Print"}]}],"kind":"FunctionDeclaration","name":"main","override":false,"params":["args"],"static":false}]}'

output=$("$NYASH_BIN" --dump-ast-json sample.hako 2>&1 | filter_noise | tr -d '\n')
if compare_outputs "$expected" "$output" "parser_ast_json_if_else_vm"; then
  test_pass parser_ast_json_if_else_vm
else
  test_fail parser_ast_json_if_else_vm
fi

cd /
rm -rf "$TEST_DIR"

