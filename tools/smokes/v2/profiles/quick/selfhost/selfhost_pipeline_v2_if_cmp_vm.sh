#!/bin/bash
# selfhost_pipeline_v2_if_cmp_vm.sh — Pipeline V2: Stage‑1 If(Compare) → MIR(JSON) → Mini‑VM

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Experimental guard: run only when explicitly enabled (two gates)
if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi
if [[ "${SMOKES_ENABLE_PIPELINE_V2_IF:-}" != "1" ]]; then
  test_skip "If(Compare) materialize is dev-only; set SMOKES_ENABLE_PIPELINE_V2_IF=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_if_cmp_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox
using "selfhost/compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    // Stage‑1 JSON: If(cond=Compare(6, 3, Gt)) { return 1 } else { return 0 }
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"If\",\"cond\":{\"type\":\"Compare\",\"op\":\"Gt\",\"lhs\":{\"type\":\"Int\",\"value\":6},\"rhs\":{\"type\":\"Int\",\"value\":3}},\"then\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":1}}],\"else\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":0}}]}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 2) // prefer_cfg=2: materialize cond copy
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_if_cmp_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
