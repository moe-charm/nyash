#!/bin/bash
# equality_string_vm.sh - Primitive string equality via Compare (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

TEST_DIR="/tmp/equality_string_vm_$$"
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

output=$(run_nyash_vm driver.nyash --dev)
compare_outputs "$expected" "$output" "equality_string_vm"

cd /
rm -rf "$TEST_DIR"

