#!/bin/bash
# userbox_unborn_field_failfast_vm.sh — Life.unborn() field write must Fail‑Fast

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_ENABLE_UNBORN_STRICT:-0}" != "1" ]; then
  log_warn "SKIP userbox_unborn_field_failfast_vm (set SMOKES_ENABLE_UNBORN_STRICT=1 to run)"
  exit 0
fi
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_CHECK_CONTRACTS=1
export NYASH_VM_USER_INSTANCE_BOXCALL=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_unborn_field_failfast_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
box Life {
  birth(n) { return 0 }
}

static box Main {
  main() {
    local p = Life.unborn()
    // Field write on unborn must fail fast
    p.name = "Alice"
    return 0
  }
}
EOF

raw_output=$("$NYASH_BIN" --backend vm "$SRC" 2>&1)
status=$?
echo "$raw_output" | sed -n '1,120p' >&2
if [ $status -ne 0 ] && echo "$raw_output" | grep -q "operation on unborn instance"; then
  log_success "userbox_unborn_field_failfast_vm emits Fail‑Fast"
  rm -rf "$TMP_DIR"; exit 0
else
  log_error "userbox_unborn_field_failfast_vm expected Fail‑Fast, got status=$status"
  rm -rf "$TMP_DIR"; exit 1
fi

