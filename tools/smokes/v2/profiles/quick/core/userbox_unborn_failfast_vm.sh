#!/bin/bash
# userbox_unborn_failfast_vm.sh — unborn instance must Fail‑Fast on operation before birth()

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_CHECK_CONTRACTS=1
export NYASH_VM_USER_INSTANCE_BOXCALL=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_unborn_failfast_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
box Life {
  birth(n) {
    return 0
  }
  nameStr() { return "OK" }
}

static box Main {
  main() {
    // Create unborn instance, then try to use it without birth()
    local alice = Life.unborn()
    // This must fail fast (unborn operation)
    print(alice.nameStr())
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$SRC")
status=$?
# Expect non-zero and specific error substring
echo "$raw_output" | sed -n '1,120p' >&2
if [ $status -ne 0 ] && echo "$raw_output" | grep -q "operation on unborn instance"; then
  log_success "userbox_unborn_failfast_vm emits Fail‑Fast"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "userbox_unborn_failfast_vm expected Fail‑Fast, got status=$status"
  rm -rf "$TMP_DIR"
  exit 1
fi
