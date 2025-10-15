#!/bin/bash
# selfhost_if_else_ret_materialize_vm.sh — If(cond=Compare) with materialization copy before branch

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
export NYASH_ALLOW_USING_FILE=1
export NYASH_DEV=1

if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

TMP_DIR="/tmp/selfhost_if_else_ret_materialize_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox
using "selfhost/compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    // Stage‑1 JSON: If(Compare(5,4) Gt) then { Return(1) } else { Return(0) }
    local ast = "{\"version\":0,\"kind\":\"Program\",\"body\":[{\"type\":\"If\",\"cond\":{\"type\":\"Compare\",\"op\":\"Gt\",\"lhs\":{\"type\":\"Int\",\"value\":5},\"rhs\":{\"type\":\"Int\",\"value\":4}},\"then\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":1}}],\"else\":[{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":0}}]}]}"
    // prefer_cfg=2 → materialize copy before branch
    local j = PipelineV2.lower_stage1_to_mir(ast, 2)
    // Debug: print JSON (for grep in smoke)
    print(j)
    return MirVmMin.run(j)
  }
}
EOF

full=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev)
out=$(echo "$full" | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_if_else_ret_materialize_vm value" || { echo "$full" | tail -n 80 >&2; cd /; rm -rf "$TMP_DIR"; exit 1; }

echo "$full" | grep -q '"op":"copy"'
if [ $? -ne 0 ]; then
  echo "$full" | tail -n 80 >&2
  log_error "selfhost_if_else_ret_materialize_vm expected a copy op in JSON"
  cd /
  rm -rf "$TMP_DIR"
  exit 1
fi

rm -rf "$TMP_DIR"
exit 0
