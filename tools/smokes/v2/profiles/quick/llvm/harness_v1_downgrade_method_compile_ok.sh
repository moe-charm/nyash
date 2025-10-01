#!/bin/bash
# tags: llvm,harness,quick,selfhost
# Purpose: JSON v1 (mir_call Method) emission downgraded to v0 for harness compile-only.

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

export NYASH_LLVM_USE_HARNESS=1
export NYASH_JSON_SCHEMA_V1=1
export NYASH_LLVM_DOWNGRADE_V1=1

TMP_DIR="/tmp/harness_v1_downgrade_method_compile_ok_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  main() {
    // Method on String literal; lowers to boxcall/v1 Method and is downgraded to v0 for harness
    local s = "abc"
    return s.length()
  }
}
EOF

out=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev)
status=$?
rm -rf "$TMP_DIR"
if [ $status -ne 0 ]; then
  test_fail "harness_v1_downgrade_method_compile_ok" "compile failed"
  exit 1
fi
test_pass "harness_v1_downgrade_method_compile_ok"
exit 0

