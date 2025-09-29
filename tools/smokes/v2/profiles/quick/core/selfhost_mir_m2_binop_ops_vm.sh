#!/bin/bash
# selfhost_mir_m2_binop_ops_vm.sh — Minimal binop ops (Add/Sub/Mul/Div/Mod)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_binop_ops_vm_$$"
mkdir -p "$TMP_DIR"

ops=(Add Sub Mul Div Mod)
# a=7, b=3 → expected: Add=10 Sub=4 Mul=21 Div=2 Mod=1
expect=(10 4 21 2 1)

for i in "${!ops[@]}"; do
  op=${ops[$i]}
  expected=${expect[$i]}
  cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin

static box Main {
  main() {
    // const 7 -> r1; const 3 -> r2; binop(OPKIND) -> r3; ret r3
    local j = MirJsonBuilderMin.make()
      |> MirJsonBuilderMin.start_module()
      |> MirJsonBuilderMin.start_function("main")
      |> MirJsonBuilderMin.start_block(0)
      |> MirJsonBuilderMin.add_const(1, 7)
      |> MirJsonBuilderMin.add_const(2, 3)
      |> MirJsonBuilderMin.add_binop("OPKIND", 1, 2, 3)
      |> MirJsonBuilderMin.add_ret(3)
      |> MirJsonBuilderMin.end_all()
      |> MirJsonBuilderMin.to_string()
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF
  sed -i "s/OPKIND/${op}/g" "$TMP_DIR/driver.nyash"
  out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
  test_name="selfhost_mir_m2_binop_${op}_vm"
  compare_outputs "$expected" "$out" "$test_name" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
done

rm -rf "$TMP_DIR"
exit 0
