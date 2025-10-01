#!/bin/bash
# exit_code_vm.sh — VM exit code unification: program return -> process exit

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/exit_code_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/ret42.nyash" << 'EOF'
static box Main { main() { return 42 } }
EOF

set +e
"$NYASH_BIN" --backend vm "$TMP_DIR/ret42.nyash" >/dev/null 2>&1
code=$?
set -e

rm -rf "$TMP_DIR"

if [ "$code" -eq 42 ]; then
  test_pass "exit_code_vm"
  exit 0
else
  test_fail "exit_code_vm" "got $code, want 42"
  exit 1
fi

