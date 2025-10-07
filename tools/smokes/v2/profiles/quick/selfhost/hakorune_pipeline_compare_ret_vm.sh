#!/bin/bash
# hakorune_pipeline_compare_ret_vm.sh — Stage‑1/2: compare→ret via FlowRunner/HakoruneVmMin

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USING=1
export NYASH_ALLOW_USING_FILE=1

# Gate: this path is sensitive to FlowRunner fast-path; enable explicitly
if [ "${SMOKES_ENABLE_STAGE12_COMPARE_RET:-0}" != "1" ]; then
  test_skip "hakorune_pipeline_compare_ret_vm" "enable with SMOKES_ENABLE_STAGE12_COMPARE_RET=1"
  exit 0
fi

TMP_DIR="/tmp/hakorune_pipeline_compare_ret_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using selfhost-compiler.pipeline_v2.flow_entry as FlowEntryBox
using selfhost.vm.boxes.mir_vm_min as MirVmMin


static box Main {
  main() {
    // Stage‑1 JSON: Return(Compare(Int 2, Int 2, Eq)) → expect 1
    local ast = "{\"type\":\"Return\",\"expr\":{\"type\":\"Compare\",\"lhs\":{\"type\":\"Int\",\"value\":2},\"rhs\":{\"type\":\"Int\",\"value\":2},\"op\":\"Eq\"}}"
    // Bypass FlowRunner fast-path; emit MIR(JSON) then execute quietly
    local j = FlowEntryBox.emit_v0_from_ast(ast, 0)
    local v = HakoruneVmEntryBox.run_min(j)
    print("" + v)
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "hakorune_pipeline_compare_ret_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
