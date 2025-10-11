#!/bin/bash
# selfhost_pipeline_v2_newbox_min_vm.sh — Pipeline V2: Stage‑1 Return(New) を MIR(JSON) に変換し、JSON形状を検証

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

TMP_DIR="/tmp/selfhost_pipeline_v2_newbox_min_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "selfhost/compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    // Return(New Counter(1,2))  → JSON形状のみ検査
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"New\",\"class\":\"Counter\",\"args\":[{\"type\":\"Int\",\"value\":1},{\"type\":\"Int\",\"value\":2}]}}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 0)
    if j.indexOf("\"op\":\"newbox\"") < 0 { print("ng1") return 1 }
    if j.indexOf("\"box_type\":\"Counter\"") < 0 { print("ng2") return 1 }
    print("ok")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_newbox_min_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

