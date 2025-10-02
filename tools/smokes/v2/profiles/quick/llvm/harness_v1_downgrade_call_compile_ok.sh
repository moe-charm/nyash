#!/bin/bash
# harness_v1_downgrade_call_compile_ok.sh — Force v1 schema ON, downgrade to v0 for harness, compile-only

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_LLVM_USE_HARNESS=1
export NYASH_JSON_SCHEMA_V1=1
export NYASH_LLVM_DOWNGRADE_V1=1

TMP_DIR="/tmp/harness_v1_downgrade_call_compile_ok_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  main() {
    // Simple program (no calls) — forces v1 schema via env, but we downgrade to v0 for harness
    return 3
  }
}
EOF

out=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev)
status=$?
rm -rf "$TMP_DIR"
if [ $status -ne 0 ] && [ $status -ne 3 ]; then
  test_fail "harness_v1_downgrade_call_compile_ok" "compile failed"
  exit 1
fi
test_pass "harness_v1_downgrade_call_compile_ok"
exit 0
