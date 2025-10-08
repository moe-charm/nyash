#!/bin/bash
# equality_box_pointer_vm.sh - Alias pointer equality must be true

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/equality_box_pointer_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
box C { v }

static box Main {
  main() {
    local a = new C()
    local b = a  // alias to same instance
    if a == b { print("true") } else { print("false") }
    return 0
  }
}
EOF

expected=$(cat << 'TXT'
true
TXT
)

output=$(run_nyash_vm driver.nyash --dev)
compare_outputs "$expected" "$output" "equality_box_pointer_vm"

cd /
rm -rf "$TEST_DIR"

