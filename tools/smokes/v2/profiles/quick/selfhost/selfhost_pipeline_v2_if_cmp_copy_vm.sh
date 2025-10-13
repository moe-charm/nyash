#!/bin/bash
# selfhost_pipeline_v2_if_cmp_copy_vm.sh — Pipeline V2: If(Compare) materialize copy insertion check

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

# Allow file-path using for pipeline boxes
export NYASH_USING=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_if_cmp_copy_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "selfhost/compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    // Stage‑1 JSON: If(cond=Compare(6, 3, Gt)) { return 1 } else { return 0 }
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"If\",\"cond\":{\"type\":\"Compare\",\"op\":\"Gt\",\"lhs\":{\"type\":\"Int\",\"value\":6},\"rhs\":{\"type\":\"Int\",\"value\":3}},\"then\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":1}}],\"else\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":0}}]}]}"
    local j = PipelineV2.lower_stage1_to_mir(ast, 2) // prefer_cfg=2: materialize cond copy
    print(j)
    return 0
  }
}
EOF

# Get last JSON line
json=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev 2>/dev/null | tr -d '\r' | awk 'match($0,/^\{.*\}$/){line=$0} END{print line}')
if ! echo "$json" | grep -q '"op":"copy"'; then
  log_error "missing materialize copy in MIR(JSON)"
  echo "$json" | sed -E 's/,/,&/g' >&2
  rm -rf "$TMP_DIR"; exit 1
fi

rm -rf "$TMP_DIR"
log_success "materialize copy present"
exit 0
