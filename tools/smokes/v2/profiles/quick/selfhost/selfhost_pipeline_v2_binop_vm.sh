#!/bin/bash
# selfhost_pipeline_v2_binop_vm.sh — Pipeline V2: Stage‑1 Return(BinOp Add) → MIR(JSON) → Mini‑VM

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

# Experimental guard: run only when explicitly enabled
if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_binop_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox
using "selfhost/compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    // Stage‑1 JSON: Return(BinOp Add(Int 7, Int 3))
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"BinOp\",\"op\":\"Add\",\"lhs\":{\"type\":\"Int\",\"value\":7},\"rhs\":{\"type\":\"Int\",\"value\":3}}}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 1)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="10"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_binop_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
