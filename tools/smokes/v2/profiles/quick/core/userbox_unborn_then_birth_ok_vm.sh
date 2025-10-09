#!/bin/bash
# userbox_unborn_then_birth_ok_vm.sh — unborn → birth() → ok

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_unborn_then_birth_ok_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
box Life {
  birth(n) { return 0 }
  nameStr() { return "OK" }
}

static box Main {
  main() {
    // unborn → birth() → then ok
    local alice = Life.unborn()
    alice.birth("Alice")
    print(alice.nameStr())
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$SRC" | grep -v '^Result: ')
echo "$raw_output" | sed -n '1,120p' >&2
result=$(echo "$raw_output" | tail -n 1 | tr -d '\r' | xargs)
if [ "$result" = "OK" ]; then
  log_success "userbox_unborn_then_birth_ok_vm prints OK"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "userbox_unborn_then_birth_ok_vm expected 'OK', got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
