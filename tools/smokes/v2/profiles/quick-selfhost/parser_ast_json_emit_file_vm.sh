#!/usr/bin/env bash
# parser_ast_json_emit_file_vm.sh — Canonical AST JSON: emit to file

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/parser_ast_json_emit_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > sample.hako << 'EOF'
function main(args) {
  print(1)
}
EOF

expected='{"kind":"Program","statements":[{"body":[{"expression":{"kind":"Literal","value":{"type":"int","value":1}},"kind":"Print"}],"kind":"FunctionDeclaration","name":"main","override":false,"params":["args"],"static":false}]}'

"$NYASH_BIN" --emit-ast-json out.json sample.hako >/dev/null 2>&1 || true
output=$(cat out.json | tr -d '\n')
if compare_outputs "$expected" "$output" "parser_ast_json_emit_file_vm"; then
  test_pass parser_ast_json_emit_file_vm
else
  test_fail parser_ast_json_emit_file_vm
fi

cd /
rm -rf "$TEST_DIR"

