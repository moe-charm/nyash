#!/bin/bash
# selfhost_pipeline_v2_stage1_neg_ws_vm.sh — Stage‑1 extract robustness: whitespace and negatives (dev)

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

TMP_DIR="/tmp/selfhost_pipeline_v2_stage1_neg_ws_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    // Case 1: Return(Int -7) with spaces
    local ast1 = "{ \n  \"version\":0, \"kind\":\"Program\", \"body\":[{ \n    \"type\":\"Return\", \n    \"expr\":{ \n      \"type\":\"Int\", \n      \"value\":   -7   \n    } \n  } ] }"
    local j1 = PipelineV2.lower_stage1_to_mir(ast1, 0)
    if j1.indexOf("\"value\":-7") < 0 { print("ng1") return 1 }

    // Case 2: Return(BinOp Add(-5, 2)) with spaces
    local ast2 = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"BinOp\",\"op\":\"Add\",\"lhs\":{\"type\":\"Int\",\"value\":  -5},\"rhs\":{\"type\":\"Int\",\"value\":  2 }}}]}"
    local j2 = PipelineV2.lower_stage1_to_mir(ast2, 0)
    if j2.indexOf("\"value\":-5") < 0 { print("ng2") return 1 }
    if j2.indexOf("\"value\":2") < 0 { print("ng3") return 1 }

    // Case 3: Return(Compare Lt(-1,0)) prefer_cfg=1 with spaces
    local ast3 = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Compare\",\"op\":\"Lt\",\"lhs\":{\"type\":\"Int\",\"value\":  -1 },\"rhs\":{\"type\":\"Int\",\"value\": 0}}}]}"
    local j3 = PipelineV2.lower_stage1_to_mir(ast3, 1)
    if j3.indexOf("\"value\":-1") < 0 { print("ng4") return 1 }
    if j3.indexOf("\"value\":0") < 0 { print("ng5") return 1 }

    print("ok")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_stage1_neg_ws_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
