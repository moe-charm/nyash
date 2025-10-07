#!/bin/bash
# selfhost_mir_m2_no_ret_fallback_vm.sh — No ret → fallback to first const i64
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_no_ret_fallback_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // No ret present; expect fallback to first const i64 (42)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j = j + "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":42}},"
    j = j + "{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":7}}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="42"
compare_outputs "$expected" "$out" "selfhost_mir_m2_no_ret_fallback_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
