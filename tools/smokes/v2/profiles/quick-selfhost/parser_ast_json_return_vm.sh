#!/usr/bin/env bash
# parser_ast_json_return_vm.sh — Canonical AST JSON: simple return

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/parser_ast_json_return_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > sample.hako << 'EOF'
function main(args) {
  return 0
}
EOF

# Canonical order for FunctionDeclaration: body,kind,name,override,params,static
# Canonical order for Return: kind,value
expected='{"kind":"Program","statements":[{"body":[{"kind":"Return","value":{"kind":"Literal","value":{"type":"int","value":0}}}],"kind":"FunctionDeclaration","name":"main","override":false,"params":["args"],"static":false}]}'

output=$("$NYASH_BIN" --dump-ast-json sample.hako 2>&1 | filter_noise | tr -d '\n')
if compare_outputs "$expected" "$output" "parser_ast_json_return_vm"; then
  test_pass parser_ast_json_return_vm
else
  test_fail parser_ast_json_return_vm
fi

cd /
rm -rf "$TEST_DIR"

