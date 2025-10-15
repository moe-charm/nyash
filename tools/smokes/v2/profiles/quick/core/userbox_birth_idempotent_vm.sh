#!/bin/bash
# userbox_birth_idempotent_vm.sh — userbox birth is idempotent (second call is no-op)
# tags: core userbox contracts

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

# Parser may disallow calling birth() outside birth context; keep this SKIP unless explicitly enabled.
if [ "${SMOKES_ENABLE_USERBOX_BIRTH_IDEMP:-0}" != "1" ]; then
  log_warn "SKIP: userbox birth idempotent test requires parser allowance for me.birth() outside birth()"
  exit 0
fi

TMP_DIR="/tmp/userbox_birth_idempotent_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF_NY'
static box T {
  _n
  birth() {
    // If birth is idempotent at contracts layer, this body should run only once.
    me._n = (me._n == null) ? 1 : (me._n + 1)
    return 0
  }
  again() {
    me.birth()
    return me._n
  }
}

static box Main {
  main() {
    // auto-birth on new T() should set _n=1, then again() calls birth() a second time → no-op
    local t = new T()
    print(t.again())
    return 0
  }
}

EOF_NY

OUT=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$OUT" "userbox_birth_idempotent_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
