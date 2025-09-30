#!/bin/bash
# selfhost_pipeline_v2_vm_llvm.sh — VM vs LLVM harness parity for Pipeline V2 (compare + binop)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

if ! "$NYASH_BIN" --version 2>/dev/null | grep -q "features.*llvm"; then
  test_skip "LLVM backend not available in this build"; exit 0
fi

# Experimental guard: only when Pipeline V2 is enabled
if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_vm_llvm_$$"
mkdir -p "$TMP_DIR"

# Case 1: Return(Compare Gt 5,4) → expect 1
cat > "$TMP_DIR/driver_cmp.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using "apps/selfhost-compiler/pipeline_v2/pipeline.nyash" as PipelineV2

static box Main {
  main() {
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Compare\",\"op\":\"Gt\",\"lhs\":{\"type\":\"Int\",\"value\":5},\"rhs\":{\"type\":\"Int\",\"value\":4}}}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 1)
    return MirVmMin.run(j)
  }
}
EOF

output_vm=$(run_nyash_vm "$TMP_DIR/driver_cmp.nyash" --dev)
NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$TMP_DIR/driver_cmp.nyash" --dev)
compare_outputs "$output_vm" "$output_llvm" "selfhost_pipeline_v2_cmp_vm_llvm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

# Case 2: Return(BinOp Add 7,3) → expect 10
cat > "$TMP_DIR/driver_binop.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using "apps/selfhost-compiler/pipeline_v2/pipeline.nyash" as PipelineV2

static box Main {
  main() {
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"BinOp\",\"op\":\"Add\",\"lhs\":{\"type\":\"Int\",\"value\":7},\"rhs\":{\"type\":\"Int\",\"value\":3}}}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 1)
    return MirVmMin.run(j)
  }
}
EOF

output_vm=$(run_nyash_vm "$TMP_DIR/driver_binop.nyash" --dev)
NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$TMP_DIR/driver_binop.nyash" --dev)
compare_outputs "$output_vm" "$output_llvm" "selfhost_pipeline_v2_binop_vm_llvm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

