#!/bin/bash
# selfhost_mir_m2_div_mod_zero_vm.sh — Div/Mod by zero return 0 (two-line check)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
if [ "${SMOKES_SELFHOST_M2M3_ENABLE:-0}" != "1" ]; then test_skip "selfhost M2/M3 gated (set SMOKES_SELFHOST_M2M3_ENABLE=1)"; exit 0; fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_div_mod_zero_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // div by zero → 0; mod by zero → 0
    local j1 = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j1 = j1 + "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":5}},"
    j1 = j1 + "{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":0}},"
    j1 = j1 + "{\"op\":\"binop\",\"dst\":3,\"op_kind\":\"Div\",\"lhs\":1,\"rhs\":2},"
    j1 = j1 + "{\"op\":\"ret\",\"value\":3}]}]}]}"
    local v1 = MirVmMin._run_min(j1)
    print(MiniVmEntryBox.int_to_str(v1))

    local j2 = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j2 = j2 + "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":5}},"
    j2 = j2 + "{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":0}},"
    j2 = j2 + "{\"op\":\"binop\",\"dst\":3,\"op_kind\":\"Mod\",\"lhs\":1,\"rhs\":2},"
    j2 = j2 + "{\"op\":\"ret\",\"value\":3}]}]}]}"
    local v2 = MirVmMin._run_min(j2)
    print(MiniVmEntryBox.int_to_str(v2))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 2 | tr -d '\r')
expected=$'0\n0'
compare_outputs "$expected" "$out" "selfhost_mir_m2_div_mod_zero_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
