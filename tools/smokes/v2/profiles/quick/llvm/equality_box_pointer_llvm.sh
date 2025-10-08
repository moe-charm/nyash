#!/bin/bash
# equality_box_pointer_llvm.sh - Alias pointer equality must be true (LLVM harness)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/equality_box_pointer_llvm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
box C { v }

static box Main {
  main() {
    local a = new C()
    local b = a
    if a == b { print("true") } else { print("false") }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
true
TXT
)

output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
if [ -z "$output" ]; then
  log_warn "LLVM backend/harness not available; skipping equality_box_pointer_llvm"
else
  compare_outputs "$expected" "$output" "equality_box_pointer_llvm"
fi

cd /
rm -rf "$TEST_DIR"

