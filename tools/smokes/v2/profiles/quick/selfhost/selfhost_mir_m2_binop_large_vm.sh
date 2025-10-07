#!/bin/bash
# selfhost_mir_m2_binop_large_vm.sh — binop with larger numbers (safe ranges)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2


export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_binop_large_vm_$$"
mkdir -p "$TMP_DIR"

# Tests
# 1) Add: 1_000_000_000 + 2_000_000_000 = 3_000_000_000
# 2) Sub: 2_000_000_000 - 1_000_000_000 = 1_000_000_000
# 3) Mul: 10_000 * 30_000 = 300_000_000

nums=("Add 1000000000 2000000000 3000000000" \
      "Sub 2000000000 1000000000 1000000000" \
      "Mul 10000 30000 300000000")

for spec in "${nums[@]}"; do
  set -- $spec
  kind=$1; a=$2; b=$3; expected=$4
  cat > "$TMP_DIR/driver_${kind}.nyash" << EOF
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j = j + "{\\"op\\":\\"const\\",\\"dst\\":1,\\"value\\":{\\"type\\":\\"i64\\",\\"value\\":${a}}},"
    j = j + "{\\"op\\":\\"const\\",\\"dst\\":2,\\"value\\":{\\"type\\":\\"i64\\",\\"value\\":${b}}},"
    j = j + "{\\"op\\":\\"binop\\",\\"op_kind\\":\\"${kind}\\",\\"lhs\\":1,\\"rhs\\":2,\\"dst\\":3},"
    j = j + "{\\"op\\":\\"ret\\",\\"value\\":3}] }]}]}"
    local v = MirVmMin._run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF
  out=$(run_nyash_vm "$TMP_DIR/driver_${kind}.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
  test_name="selfhost_mir_m2_binop_large_${kind}_vm"
  compare_outputs "$expected" "$out" "$test_name" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
done

rm -rf "$TMP_DIR"
exit 0
