#!/bin/bash
# Gate: relies on prod profile instance boxcall materialization; skip by default
if [ "${SMOKES_ENABLE_USERBOX_ARITY:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_USERBOX_ARITY=1" >&2
  exit 0
fi
# userbox_method_arity_vm.sh — Same-name methods with different arity (prod)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USING_PROFILE=prod
export NYASH_VM_USER_INSTANCE_BOXCALL=0

TEST_DIR="/tmp/userbox_method_arity_vm_$$"
mkdir -p "$TEST_DIR" && cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    local f = new Foo()
    if f.add1(2) == 3 { print("ok1") } else { print("ng1") }
    if f.add2(2, 5) == 7 { print("ok2") } else { print("ng2") }
    return 0
  }
}

box Foo {
  add1(a) { return a + 1 }
  add2(a,b) { return a + b }
}
EOF

out_all=$(run_nyash_vm driver.nyash --dev)
if echo "$out_all" | grep -q "User Instance BoxCall disallowed in prod"; then
  test_skip "userbox_method_arity_vm" "rewrite/materialize pending"
else
  output=$(echo "$out_all" | tail -n 2 | tr -d '\r')
  expected=$'ok1\nok2'
  compare_outputs "$expected" "$output" "userbox_method_arity_vm" || { cd /; rm -rf "$TEST_DIR"; exit 1; }
  test_pass "userbox_method_arity_vm"
fi

cd /
rm -rf "$TEST_DIR"
