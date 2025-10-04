#!/bin/bash
# selfhost_mir_m2_compare_neg_binop_vm.sh — compare with negative via binop (3-7=-4) vs 0

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2


export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_compare_neg_binop_vm_$$"
mkdir -p "$TMP_DIR"

cases=(Lt Ge)
expects=(1 0)

for i in "${!cases[@]}"; do
  op=${cases[$i]}
  expected=${expects[$i]}
  cat > "$TMP_DIR/driver_${op}.nyash" << EOF
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // r1=3, r2=7, r3=r1-r2=-4; r4=0; compare(${op}) r3 vs r4
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j = j + "{\\"op\\":\\"const\\",\\"dst\\":1,\\"value\\":{\\"type\\":\\"i64\\",\\"value\\":3}},"
    j = j + "{\\"op\\":\\"const\\",\\"dst\\":2,\\"value\\":{\\"type\\":\\"i64\\",\\"value\\":7}},"
    j = j + "{\\"op\\":\\"binop\\",\\"op_kind\\":\\"Sub\\",\\"lhs\\":1,\\"rhs\\":2,\\"dst\\":3},"
    j = j + "{\\"op\\":\\"const\\",\\"dst\\":4,\\"value\\":{\\"type\\":\\"i64\\",\\"value\\":0}},"
    j = j + "{\\"op\\":\\"compare\\",\\"cmp\\":\\"${op}\\",\\"lhs\\":3,\\"rhs\\":4,\\"dst\\":5},"
    j = j + "{\\"op\\":\\"ret\\",\\"value\\":5}] }]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF
  out=$(run_nyash_vm "$TMP_DIR/driver_${op}.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
  test_name="selfhost_mir_m2_compare_neg_${op}_vm"
  compare_outputs "$expected" "$out" "$test_name" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
done

rm -rf "$TMP_DIR"
exit 0
