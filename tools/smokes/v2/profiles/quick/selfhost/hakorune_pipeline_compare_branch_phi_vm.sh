#!/bin/bash
# hakorune_pipeline_compare_branch_phi_vm.sh — Stage‑1/2: compare→branch→phi via FlowRunner/HakoruneVmMin

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_ENABLE_USING=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/hakorune_pipeline_compare_branch_phi_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using selfhost.vm.flow_runner as FlowRunner

static box Main {
  main() {
    // Stage‑1 JSON: If(Compare(7 Gt 2)) { Return(1) } else { Return(0) } — prefer CFG→materialize
    local ast = "{\"type\":\"If\",\"condition\":{\"type\":\"Compare\",\"lhs\":{\"type\":\"Int\",\"value\":7},\"rhs\":{\"type\":\"Int\",\"value\":2},\"op\":\"Gt\"},\"then\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":1}}],\"else\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":0}}]}"
    // prefer_cfg=2 to materialize compare and route through branch/phi
    local v = FlowRunner.run_vm_min_from_ast(ast, 2, 1)
    print("" + v)
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "hakorune_pipeline_compare_branch_phi_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
