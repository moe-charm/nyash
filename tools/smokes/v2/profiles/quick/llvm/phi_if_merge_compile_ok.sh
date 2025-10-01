#!/bin/bash
# phi_if_merge_compile_ok.sh — LLVM harness compile-only on If-merge (PHI expected)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_LLVM_USE_HARNESS=1
export NYASH_LLVM_PHI_STRICT=1

TMP_DIR="/tmp/phi_if_merge_compile_ok_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  main() {
    local x = 0
    if (1 < 2) { x = 1 } else { x = 2 }
    return x
  }
}
EOF

out=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev)
status=$?
rm -rf "$TMP_DIR"
if [ $status -ne 0 ]; then
  test_fail "phi_if_merge_compile_ok" "harness compile failed" && exit 1
fi
test_pass "phi_if_merge_compile_ok" && exit 0

