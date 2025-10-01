#!/bin/bash
# selfhost_pipeline_v2_call_args_boundary_vm.sh — Stage‑1 Return(Call) with negatives/whitespace/strings; Mini‑VM exec

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

TMP_DIR="/tmp/selfhost_pipeline_v2_call_args_boundary_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/pipeline_v2/pipeline.hako" as PipelineV2
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Return(Call "Mix"(-3, "x", 5)) → extractor should accept -3 and 5, skip string; expect 2
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Call\",\"name\":\"Mix\",\"args\":[{\"type\":\"Int\",\"value\":  -3 },{\"type\":\"String\",\"value\":\"x\"},{\"type\":\"Int\",\"value\":5}]}}]}"
    // Use v1 compat path (emits v1 then adapt to v0). This tests extractor robustness end-to-end.
    local j = PipelineV2.lower_stage1_to_mir_v1_compat(ast, 0)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="2"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_call_args_boundary_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
