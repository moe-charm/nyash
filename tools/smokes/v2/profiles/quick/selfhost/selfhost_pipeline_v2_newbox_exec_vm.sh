#!/bin/bash
# selfhost_pipeline_v2_newbox_exec_vm.sh — Pipeline V2: Return(New) → MIR(JSON v0) → Mini‑VM exec (sum of args)

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

TMP_DIR="/tmp/selfhost_pipeline_v2_newbox_exec_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Build: const 1->r1; const 2->r2; newbox Counter([1,2])->r3; ret r3; expect 3
    local b = MirJsonBuilderMin.make()
      |> MirJsonBuilderMin.start_module()
      |> MirJsonBuilderMin.start_function("main")
      |> MirJsonBuilderMin.start_block(0)
      |> MirJsonBuilderMin.add_const(1, 1)
      |> MirJsonBuilderMin.add_const(2, 2)
      |> MirJsonBuilderMin.add_newbox_range("Counter", 1, 2, 3)
      |> MirJsonBuilderMin.add_ret(3)
      |> MirJsonBuilderMin.end_all()
    local j = MirJsonBuilderMin.to_string(b)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="3"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_newbox_exec_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
