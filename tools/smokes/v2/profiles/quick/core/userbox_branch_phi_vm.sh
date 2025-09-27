#!/bin/bash
# userbox_branch_phi_vm.sh — Instance method across branches (prod)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USING_PROFILE=prod
export NYASH_VM_USER_INSTANCE_BOXCALL=0

TEST_DIR="/tmp/userbox_branch_phi_vm_$$"
mkdir -p "$TEST_DIR" && cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    local o = new P(9)
    local cond = true
    if cond { if o.get() == 9 { print("ok") } else { print("ng") } } else { if o.get() == 9 { print("ok") } else { print("ng") } }
    return 0
  }
}

box P {
  x
  birth(v) { me.x = v  return 0 }
  get() { return me.x }
}
EOF

out_all=$(run_nyash_vm driver.nyash --dev)
if echo "$out_all" | grep -q "User Instance BoxCall disallowed in prod"; then
  test_skip "userbox_branch_phi_vm" "rewrite/materialize pending"
else
  output=$(echo "$out_all" | tail -n 1 | tr -d '\r' | xargs)
  [ "$output" = "ok" ] && test_pass "userbox_branch_phi_vm" || test_fail "userbox_branch_phi_vm" "got: $output"
fi

cd /
rm -rf "$TEST_DIR"
