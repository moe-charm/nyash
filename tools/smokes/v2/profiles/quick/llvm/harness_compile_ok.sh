#!/bin/bash
# harness_compile_ok.sh — LLVM harness compile-only smoke (object emission)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_LLVM_USE_HARNESS=1

TMP_DIR="/tmp/harness_compile_ok_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  main() {
    // Minimal program: return 0 (as i64)
    return 0
  }
}
EOF

# We only assert that harness path compiles to an object file without errors.
out=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev)
status=$?
rm -rf "$TMP_DIR"
if [ $status -ne 0 ]; then
  test_fail "harness_compile_ok" "harness compile failed" && exit 1
fi
test_pass "harness_compile_ok" && exit 0

