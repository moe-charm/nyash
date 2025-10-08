#!/bin/bash
# equality_equals_recursion_vm.sh - equals 内で me == other を使っても落ちないこと

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/equality_equals_recursion_vm_$$"
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

output=$(run_nyash_vm driver.nyash --dev)
compare_outputs "$expected" "$output" "equality_equals_recursion_vm"

cd /
rm -rf "$TEST_DIR"

