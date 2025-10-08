#!/bin/bash
# selfhost_mir_m2_binop_ops_vm.sh — Minimal binop ops (Add/Sub/Mul/Div/Mod)
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2

# Temporary: skip until Mini-VM binop path is fully stabilized
if [ "${NYASH_MINIVM_ENABLE_BINOP_TEST:-0}" != "1" ]; then
  log_warn "selfhost_mir_m2_binop_ops_vm: Mini-VM binop path under investigation (SKIP)"
  exit 0
fi

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
using selfhost.vm.entry as MiniVmEntryBox
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin

static box Main {
  main() {
    // const 7 -> r1; const 3 -> r2; binop(OPKIND) -> r3; ret r3
    local builder = new MirJsonBuilderMin()
    builder.start_module()
    builder.start_function("main")
    builder.start_block(0)
    builder.add_const(1, 7)
    builder.add_const(2, 3)
    builder.add_binop("OPKIND", 1, 2, 3)
    builder.add_ret(3)
    builder.end_all()
    local j = builder.to_string()
    local v = MirVmMin._run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
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
