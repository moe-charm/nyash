#!/bin/bash
# userbox_static_call_vm.sh — Static method call on user box (prod, no VM fallback)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USING_PROFILE=prod
export NYASH_VM_USER_INSTANCE_BOXCALL=0

TEST_DIR="/tmp/userbox_static_call_vm_$$"
mkdir -p "$TEST_DIR" && cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    if MyBox.ping() == 1 { print("ok") } else { print("ng") }
    return 0
  }
}

static box MyBox { ping() { return 1 } }
EOF

output=$(run_nyash_vm driver.nyash --dev)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)
[ "$output" = "ok" ] && test_pass "userbox_static_call_vm" || test_fail "userbox_static_call_vm" "got: $output"

cd /
rm -rf "$TEST_DIR"
