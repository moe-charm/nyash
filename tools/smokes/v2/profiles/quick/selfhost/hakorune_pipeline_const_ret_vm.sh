#!/bin/bash
# hakorune_pipeline_const_ret_vm.sh — Stage‑1/2: const→ret via FlowRunner/HakoruneVmMin

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

export NYASH_USING=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/hakorune_pipeline_const_ret_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using selfhost.vm.flow_runner as FlowRunner

static box Main {
  main() {
    // Minimal Stage‑1 JSON: Return(Int 7)
    local ast = "{\"type\":\"Return\",\"expr\":{\"type\":\"Int\",\"value\":7}}"
    local v = FlowRunner.run_vm_min_from_ast(ast, 0, 1)
    print("" + v)
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="7"
compare_outputs "$expected" "$out" "hakorune_pipeline_const_ret_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
