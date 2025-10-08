#!/bin/bash
# equality_box_vm.sh - Box equality via op_eq path (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/equality_box_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
box Point {
  x
  y
  birth(a, b) {
    me.x = a
    me.y = b
  }
  equals(other) { return me.x == other.x and me.y == other.y }
}

box Simple {
  v
  birth(val) {
    me.v = val
  }
  equals(other) {
    return me.v == other.v
  }
}

static box Main {
  main() {
    local p1 = new Point(3, 4)
    local p2 = new Point(3, 4)
    if p1 == p2 { print("true") } else { print("false") }

    local s1 = new Simple(1)
    local s2 = new Simple(2)
    if s1 == s2 { print("false") } else { print("true") }

    local s3 = new Simple(5)
    local s4 = new Simple(5)
    if s3 == s4 { print("true") } else { print("false") }

    if 42 == 42 { print("true") } else { print("false") }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
true
true
true
true
TXT
)

output=$(run_nyash_vm driver.nyash --dev)
compare_outputs "$expected" "$output" "equality_box_vm"

cd /
rm -rf "$TEST_DIR"
