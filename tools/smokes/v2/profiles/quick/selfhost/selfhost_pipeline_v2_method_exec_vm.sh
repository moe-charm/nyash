#!/bin/bash
# selfhost_pipeline_v2_method_exec_vm.sh — Pipeline V2: Return(Method) → MIR(JSON v0) → Mini‑VM exec (sum of args, ignore recv)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_method_exec_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // Build: const recv=0 -> r1; const 3->r2; const 4->r3; boxcall sum2(recv=1,args=[2,3])->r4; ret r4; expect 7
    local b = new MirJsonBuilderMin()
    b.start_module()
    b.start_function("main")
    b.start_block(0)
    b.add_const(1, 0)
    b.add_const(2, 3)
    b.add_const(3, 4)
    b.add_boxcall_range("sum2", 1, 2, 2, 4)
    b.add_ret(4)
    b.end_all()
    local j = b.to_string()
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="7"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_method_exec_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
