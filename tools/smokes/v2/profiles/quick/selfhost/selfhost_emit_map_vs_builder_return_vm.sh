#!/bin/bash
# selfhost_emit_map_vs_builder_return_vm.sh — Compare Map/Array stringify vs Builder output (return int)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_emit_map_vs_builder_return_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "selfhost/compiler/pipeline_v2/emit_mir_flow.hako" as EmitMirFlow
using "selfhost/compiler/pipeline_v2/emit_mir_flow_map.hako" as EmitMirFlowMap

static box Main {
  main() {
    local a = EmitMirFlow.emit_return_int(42)
    local b = EmitMirFlowMap.emit_return_int(42)
    // Shape-based equality (key order may differ by stringify policy)
    if a.indexOf("\"op\":\"const\"") < 0 { print("ng") return 1 }
    if b.indexOf("\"op\":\"const\"") < 0 { print("ng") return 1 }
    if a.indexOf("\"value\":42") < 0 { print("ng") return 1 }
    if b.indexOf("\"value\":42") < 0 { print("ng") return 1 }
    if a.indexOf("\"op\":\"ret\"") < 0 { print("ng") return 1 }
    if b.indexOf("\"op\":\"ret\"") < 0 { print("ng") return 1 }
    print("ok")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "selfhost_emit_map_vs_builder_return_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
