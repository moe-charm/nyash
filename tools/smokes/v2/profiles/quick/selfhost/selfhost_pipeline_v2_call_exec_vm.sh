#!/bin/bash
# selfhost_pipeline_v2_call_exec_vm.sh — Pipeline V2: Return(Call) → MIR(JSON v0) → Mini‑VM exec (sum of args)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Experimental guard
if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_call_exec_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Build: const 5, const 7, call Add2([1,2]) -> r3, ret r3; expect 12
    local b = MirJsonBuilderMin.make()
      |> MirJsonBuilderMin.start_module()
      |> MirJsonBuilderMin.start_function("main")
      |> MirJsonBuilderMin.start_block(0)
      |> MirJsonBuilderMin.add_const(1, 5)
      |> MirJsonBuilderMin.add_const(2, 7)
      |> MirJsonBuilderMin.add_call_range("Add2", 1, 2, 3)
      |> MirJsonBuilderMin.add_ret(3)
      |> MirJsonBuilderMin.end_all()
    local j = MirJsonBuilderMin.to_string(b)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="12"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_call_exec_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
