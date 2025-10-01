#!/bin/bash
# selfhost_mir_m2_compare_ops_vm.sh — Minimal compare ops (Eq/Ne/Lt/Le/Gt/Ge) → 0/1
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_compare_ops_vm_$$"
mkdir -p "$TMP_DIR"

ops=(Eq Ne Lt Le Gt Ge)
# a=5, b=4 → expected: Eq=0 Ne=1 Lt=0 Le=0 Gt=1 Ge=1
expect=(0 1 0 0 1 1)

for i in "${!ops[@]}"; do
  op=${ops[$i]}
  expected=${expect[$i]}
  cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin

static box Main {
  main() {
    // const 5 -> r1; const 4 -> r2; compare(OPKIND) -> r3; ret r3
    local j = MirJsonBuilderMin.make()
      |> MirJsonBuilderMin.start_module()
      |> MirJsonBuilderMin.start_function("main")
      |> MirJsonBuilderMin.start_block(0)
      |> MirJsonBuilderMin.add_const(1, 5)
      |> MirJsonBuilderMin.add_const(2, 4)
      |> MirJsonBuilderMin.add_compare("OPKIND", 1, 2, 3)
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
  test_name="selfhost_mir_m2_compare_${op}_vm"
  compare_outputs "$expected" "$out" "$test_name" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
done

rm -rf "$TMP_DIR"
exit 0
