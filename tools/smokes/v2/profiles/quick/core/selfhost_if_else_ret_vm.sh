#!/bin/bash
# selfhost_if_else_ret_vm.sh — Pipeline V2: minimal if/else → branch/jump/ret（設計先行・dev）

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
export NYASH_ALLOW_USING_FILE=1
export NYASH_DEV=1

if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

TMP_DIR="/tmp/selfhost_if_else_ret_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using "apps/selfhost-compiler/pipeline_v2/pipeline.nyash" as PipelineV2

static box Main {
  main() {
    // Stage‑1 JSON: If(Compare(5,4) Gt) then { Return(1) } else { Return(0) }
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"If\",\"cond\":{\"type\":\"Compare\",\"op\":\"Gt\",\"lhs\":{\"type\":\"Int\",\"value\":5},\"rhs\":{\"type\":\"Int\",\"value\":4}},\"then\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":1}}],\"else\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":0}}]}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 1)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_if_else_ret_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
