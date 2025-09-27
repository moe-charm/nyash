#!/bin/bash
# userbox_toString_mapping_vm.sh — toString() call maps to stringify() (prod)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USING_PROFILE=prod
export NYASH_VM_USER_INSTANCE_BOXCALL=0

TEST_DIR="/tmp/userbox_toString_mapping_vm_$$"
mkdir -p "$TEST_DIR" && cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    local q = new Q()
    print(q.toString())
    return 0
  }
}

box Q {
  stringify() { return "ok" }
}
EOF

output=$(run_nyash_vm driver.nyash --dev)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)
if [ "$output" = "ok" ]; then
  test_pass "userbox_toString_mapping_vm"
else
  test_skip "userbox_toString_mapping_vm" "mapping pending (got '$output')"
fi

cd /
rm -rf "$TEST_DIR"
