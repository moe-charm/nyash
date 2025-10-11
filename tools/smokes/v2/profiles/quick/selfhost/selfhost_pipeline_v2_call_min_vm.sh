#!/bin/bash
# selfhost_pipeline_v2_call_min_vm.sh — Pipeline V2: Stage‑1 Return(Call) を MIR(JSON) に変換し、JSONの形を検証

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Experimental guard: run only when explicitly enabled
if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_call_min_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "selfhost/compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    // Stage‑1 JSON: Return(Call name:"Add2", args:[Int 5, Int 7])
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Call\",\"name\":\"Add2\",\"args\":[{\"type\":\"Int\",\"value\":5},{\"type\":\"Int\",\"value\":7}]}}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 0)
    // 形だけを検証（Mini‑VM には call 実行器が無い）
    if j.indexOf("\"op\":\"call\"") < 0 { print("ng1") return 1 }
    if j.indexOf("\"name\":\"Add2\"") < 0 { print("ng2") return 1 }
    // Accept any whitespace/formatting: only assert that an args array exists
    if j.indexOf("\"args\":") < 0 { print("ng3") return 1 }
    print("ok")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_call_min_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
