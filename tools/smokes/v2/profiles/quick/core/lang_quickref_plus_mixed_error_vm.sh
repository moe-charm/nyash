#!/bin/bash
# lang_quickref_plus_mixed_error_vm.sh — '+' mixed types should error (planned)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"
require_env || exit 2
preflight_plugins || exit 2

# Enable '+' mixed guard for this test
export NYASH_PLUS_MIX_ERROR=1

TMP_DIR="/tmp/lang_quickref_plus_mixed_error_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/code.nyash" << 'EOF'
static box Main { main(){ local x; x = 1 + "s"; print(x); return 0 } }
EOF

if check_error_pattern "$TMP_DIR/code.nyash" "Type error|Invalid|cannot" "lang_quickref_plus_mixed_error_vm"; then
  rm -rf "$TMP_DIR"; exit 0
else
  rm -rf "$TMP_DIR"; exit 1
fi
