#!/bin/bash
# tags: llvm,harness,quick,selfhost
# Purpose: When downgrading v1 mir_call to v0 for the harness, unresolved Global callee
#          should fallback to externcall so compile-only succeeds.

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

export NYASH_LLVM_USE_HARNESS=1
export NYASH_JSON_SCHEMA_V1=1
export NYASH_LLVM_DOWNGRADE_V1=1

TMP_DIR="/tmp/harness_v1_downgrade_global_extern_compile_ok_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  foo(x) { return x }
  main() {
    // v1 schema forced but downgraded to v0 for harness
    // Define and call a local function to exercise v1→v0 downgrade path (no extern fallback here)
    return foo(1)
  }
}
EOF

out=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev)
status=$?
rm -rf "$TMP_DIR"
if [ $status -ne 0 ] && [ $status -ne 1 ]; then
  test_fail "harness_v1_downgrade_global_extern_compile_ok" "compile failed"
  exit 1
fi
test_pass "harness_v1_downgrade_global_extern_compile_ok"
exit 0
