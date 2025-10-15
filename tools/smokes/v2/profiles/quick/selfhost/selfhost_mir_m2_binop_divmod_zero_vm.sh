#!/bin/bash
# selfhost_mir_m2_binop_divmod_zero_vm.sh — Div/Mod by zero boundary behavior (expect 0)

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

TMP_DIR="/tmp/selfhost_mir_m2_binop_divmod_zero_vm_$$"
mkdir -p "$TMP_DIR"

ops=(Div Mod)
expect=(0 0)

for i in "${!ops[@]}"; do
  op=${ops[$i]}
  expected=${expect[$i]}
  cat > "$TMP_DIR/driver.nyash" << EOF
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // const 7 -> r1; const 0 -> r2; binop(${op}) -> r3; ret r3
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j = j + "{\\"op\\":\\"const\\",\\"dst\\":1,\\"value\\":{\\"type\\":\\"i64\\",\\"value\\":7}},"
    j = j + "{\\"op\\":\\"const\\",\\"dst\\":2,\\"value\\":{\\"type\\":\\"i64\\",\\"value\\":0}},"
    j = j + "{\\"op\\":\\"binop\\",\\"op_kind\\":\\"${op}\\",\\"lhs\\":1,\\"rhs\\":2,\\"dst\\":3},"
    j = j + "{\\"op\\":\\"ret\\",\\"value\\":3}] }]}]}"
    local v = MirVmMin._run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF
  out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
  test_name="selfhost_mir_m2_binop_${op}_zero_vm"
  compare_outputs "$expected" "$out" "$test_name" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
done

rm -rf "$TMP_DIR"
exit 0
