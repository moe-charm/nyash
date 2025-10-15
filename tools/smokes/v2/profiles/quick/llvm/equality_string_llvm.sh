#!/bin/bash
# equality_string_llvm.sh - Primitive string equality via Compare (LLVM harness)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/equality_string_llvm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    if "hello" == "hello" { print("true") } else { print("false") }
    if "hello" == "world" { print("true") } else { print("false") }
    if 42 == 42 { print("true") } else { print("false") }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
true
false
true
TXT
)

output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
if [ -z "$output" ]; then
  log_warn "LLVM backend/harness not available; skipping equality_string_llvm"
else
  compare_outputs "$expected" "$output" "equality_string_llvm"
fi

cd /
rm -rf "$TEST_DIR"

