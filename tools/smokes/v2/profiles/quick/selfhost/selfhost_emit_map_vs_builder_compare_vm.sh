#!/bin/bash
# selfhost_emit_map_vs_builder_compare_vm.sh — Compare Map/Array stringify vs Builder output (compare cfg)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_emit_map_vs_builder_compare_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "selfhost/compiler/pipeline_v2/emit_mir_flow.hako" as EmitMirFlow
using "selfhost/compiler/pipeline_v2/emit_mir_flow_map.hako" as EmitMirFlowMap

static box Main {
  main() {
    // materialize=0 の compare cfg を比較
    local a = EmitMirFlow.emit_compare_cfg2(5, 4, "Gt", 0)
    local b = EmitMirFlowMap.emit_compare_cfg2(5, 4, "Gt", 0)
    if a.indexOf("\"op\":\"compare\"") < 0 { print("ng") return 1 }
    if b.indexOf("\"op\":\"compare\"") < 0 { print("ng") return 1 }
    if a.indexOf("\"op\":\"branch\"") < 0 { print("ng") return 1 }
    if b.indexOf("\"op\":\"branch\"") < 0 { print("ng") return 1 }
    if a.indexOf("\"op\":\"ret\"") < 0 { print("ng") return 1 }
    if b.indexOf("\"op\":\"ret\"") < 0 { print("ng") return 1 }
    print("ok")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "selfhost_emit_map_vs_builder_compare_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
