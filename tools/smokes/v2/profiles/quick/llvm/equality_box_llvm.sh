#!/bin/bash
# equality_box_llvm.sh - Box equality via op_eq path (LLVM harness)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/equality_box_llvm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Ensure harness resolvable
mkdir -p tools
cp -f "$NYASH_ROOT/tools/llvmlite_harness.py" tools/ 2>/dev/null || true

cat > driver.nyash << 'EOF'
box Point {
  x, y
  birth(a, b) { me.x = a; me.y = b }
  equals(other) { return me.x == other.x && me.y == other.y }
}

box Simple { v }

static box Main {
  main() {
    local p1 = new Point(3, 4)
    local p2 = new Point(3, 4)
    if p1 == p2 { print("true") } else { print("false") }

    local s1 = new Simple()
    local s2 = new Simple()
    if s1 == s2 { print("true") } else { print("false") }

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
compare_outputs "$expected" "$output" "equality_box_llvm"

cd /
rm -rf "$TEST_DIR"
