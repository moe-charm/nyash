#!/bin/bash
# userbox_unborn_call_failfast_vm.sh — call("Life.nameStr/1", p) on unborn must Fail‑Fast

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_ENABLE_UNBORN_STRICT:-0}" != "1" ]; then
  log_warn "SKIP userbox_unborn_call_failfast_vm (set SMOKES_ENABLE_UNBORN_STRICT=1 to run)"
  exit 0
fi
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_CHECK_CONTRACTS=1
export NYASH_VM_USER_INSTANCE_BOXCALL=0
# Macro overlay for call("...") normalization
export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros:self"
export NYASH_SYNTAX_SUGAR_LEVEL=full

require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_unborn_call_failfast_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
box Life {
  birth(n) { return 0 }
  nameStr() { return "OK" }
}

static box Main {
  main() {
    local p = Life.unborn()
    // Use macro call() to dispatch via ModuleFunction name, receiver as arg
    print(call("Life.nameStr/1", p))
    return 0
  }
}
EOF

raw_output=$("$NYASH_BIN" --backend vm "$SRC" 2>&1)
status=$?
echo "$raw_output" | sed -n '1,160p' >&2
if [ $status -ne 0 ] && echo "$raw_output" | grep -q "operation on unborn instance"; then
  log_success "userbox_unborn_call_failfast_vm emits Fail‑Fast"
  rm -rf "$TMP_DIR"; exit 0
else
  log_error "userbox_unborn_call_failfast_vm expected Fail‑Fast, got status=$status"
  rm -rf "$TMP_DIR"; exit 1
fi

