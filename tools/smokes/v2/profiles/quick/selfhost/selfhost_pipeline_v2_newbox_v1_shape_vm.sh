#!/bin/bash
# selfhost_pipeline_v2_newbox_v1_shape_vm.sh — Pipeline V2 v1(Constructor) shape (VM-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

if [[ "${SMOKES_ENABLE_JSON_V1:-0}" != "1" ]]; then
  test_skip "JSON v1 path disabled; set SMOKES_ENABLE_JSON_V1=1 to run"
  exit 0
fi

if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_newbox_v1_shape_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"New\",\"class\":\"BoxT\",\"args\":[{\"type\":\"Int\",\"value\":3}]}}]}"
    local j = PipelineV2.lower_stage1_to_mir_v1(ast, 0)
    if j.indexOf("\"op\":\"mir_call\"") < 0 { print("ng1") return 1 }
    if j.indexOf("\"type\":\"Constructor\"") < 0 { print("ng2") return 1 }
    print("ok")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
compare_outputs "ok" "$out" "selfhost_pipeline_v2_newbox_v1_shape_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

