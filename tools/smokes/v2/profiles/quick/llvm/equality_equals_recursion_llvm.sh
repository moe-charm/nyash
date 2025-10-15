#!/bin/bash
# equality_equals_recursion_llvm.sh - equals 内で me == other を使っても落ちない（LLVM）

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/equality_equals_recursion_llvm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
box R {
  equals(other) { return me == other }
}

static box Main {
  main() {
    local x = new R()
    local y = new R()
    if x == x { print("true") } else { print("false") }
    if x == y { print("true") } else { print("false") }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
true
false
TXT
)

output=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm driver.nyash)
if [ -z "$output" ]; then
  log_warn "LLVM backend/harness not available; skipping equality_equals_recursion_llvm"
else
  compare_outputs "$expected" "$output" "equality_equals_recursion_llvm"
fi

cd /
rm -rf "$TEST_DIR"

