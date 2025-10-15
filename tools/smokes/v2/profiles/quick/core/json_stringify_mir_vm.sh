#!/bin/bash
# json_stringify_mir_vm.sh — Build MIR via Map/Array literals + JSON.stringify_map; execute on Mini‑VM

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Experimental: enable explicitly
if [[ "${NYASH_JSON_STRINGIFY_DEV:-}" != "1" ]]; then
  test_skip "JSON.stringify(Map/Array) Ny-impl is experimental; set NYASH_JSON_STRINGIFY_DEV=1 to run"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/json_stringify_mir_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox
using "apps/lib/json_native/stringify.hako" as JSON

static box Main {
  main() {
    // MIR object (Map/Array literals)
    local mir = {
      functions: [{
        name: "main",
        params: [],
        blocks: [{
          id: 0,
          instructions: [
            { op:"const", dst:1, value:{ type:"i64", value:7 } },
            { op:"const", dst:2, value:{ type:"i64", value:4 } },
            { op:"compare", cmp:"Gt", lhs:1, rhs:2, dst:3 },
            { op:"ret", value:3 }
          ]
        }]
      }]
    }
    local j = JSON.stringify_map(mir)
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "json_stringify_mir_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
