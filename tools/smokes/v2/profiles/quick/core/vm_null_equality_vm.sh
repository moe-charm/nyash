#!/bin/bash
# vm_null_equality_vm.sh — Repro: null equality and stringification

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Disabled by default (diagnostic repro). Enable with SMOKES_ENABLE_VM_REPRO=1
if [ "${SMOKES_ENABLE_VM_REPRO:-0}" != "1" ]; then
  test_skip "vm_null_equality_vm (diagnostic repro)" \
    "Enable with SMOKES_ENABLE_VM_REPRO=1 to run"
  exit 0
fi

TMP_DIR="/tmp/vm_null_equality_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Probe {
  produce_null() {
    return null
  }
}

static box Main {
  main() {
    local x = Probe.produce_null()
    print("is_null==" + (x == null ? "1" : "0"))
    print("not_null==" + (x != null ? "1" : "0"))
    print("str='" + ("" + x) + "'")
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$TMP_DIR/driver.nyash")
is_null=$(echo "$raw_output" | sed -n 's/^is_null==\([01]\)$/\1/p')
not_null=$(echo "$raw_output" | sed -n 's/^not_null==\([01]\)$/\1/p')
strval=$(echo "$raw_output" | sed -n "s/^str='\(.*\)'/\1/p")

# Diagnostics: print raw lines for analysis
echo "$raw_output" | sed -n '1,120p' >&2

# Optional strict assertion when SMOKES_ASSERT=1
if [ "${SMOKES_ASSERT:-0}" = "1" ]; then
  if [ "$is_null" != "1" ] || [ "$not_null" != "0" ] || [ "$strval" != "" ]; then
    log_error "vm_null_equality_vm assertion failed (enable-only)"
    exit 1
  fi
fi

log_success "vm_null_equality_vm ran (diagnostic)"
rm -rf "$TMP_DIR"
exit 0
