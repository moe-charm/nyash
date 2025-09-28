#!/bin/bash
# lang_quickref_asi_error_vm.sh — ASI line continuation after binop should error (planned)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"
require_env || exit 2
preflight_plugins || exit 2

# Enable strict ASI guard for this test
export NYASH_ASI_STRICT=1

TMP_DIR="/tmp/lang_quickref_asi_error_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/code.nyash" << 'EOF'
static box Main {
  main(){
    print(1 +
      2)
    return 0
  }
}
EOF

# Expect parse error mentioning unexpected newline/continuation
if check_error_pattern "$TMP_DIR/code.nyash" "Parse error|Tokenize error|Unexpected" "lang_quickref_asi_error_vm"; then
  rm -rf "$TMP_DIR"; exit 0
else
  rm -rf "$TMP_DIR"; exit 1
fi
