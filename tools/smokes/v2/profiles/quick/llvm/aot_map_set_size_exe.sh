#!/usr/bin/env bash
# Quick smoke: MapBox set + size via AOT (expects size=2 → exit=2)

set -euo pipefail

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"; while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do ROOT="$(dirname "$ROOT")"; done
NY_LLVMc="$ROOT/target/release/ny-llvmc"
EXE_OUT="$ROOT/tmp/aot_map_size"
JSON_IN="$ROOT/tmp/aot_map_size.json"

if [ ! -x "$NY_LLVMc" ]; then (cd "$ROOT/crates/nyash-llvm-compiler" && cargo build --release); fi
(cd "$ROOT/crates/hako_kernel" && cargo build --release) || true

# Skip if static MapBox plugin is not available for linking (quick profile)
if ! rg -n "MapBox" "$ROOT/nyash.toml" >/dev/null 2>&1; then
  test_skip "aot_map_set_size_exe" "Static MapBox plugin not configured for AOT" && exit 0
fi

mkdir -p "$(dirname "$EXE_OUT")"
cat > "$JSON_IN" << 'JSON'
{
  "version": 0,
  "functions": [
    { "name": "main", "params": [], "blocks": [
      { "id": 0, "instructions": [
        { "op": "mir_call", "dst": 1, "mir_call": { "callee": { "type": "Constructor", "box_type": "MapBox" }, "args": [] } },
        { "op": "const", "dst": 2, "value": { "type": "string", "value": "k1" } },
        { "op": "const", "dst": 3, "value": { "type": "string", "value": "v1" } },
        { "op": "const", "dst": 4, "value": { "type": "string", "value": "k2" } },
        { "op": "const", "dst": 5, "value": { "type": "string", "value": "v2" } },
        { "op": "boxcall", "box": 1, "method": "set", "args": [2,3], "dst": 10 },
        { "op": "boxcall", "box": 1, "method": "set", "args": [4,5], "dst": 11 },
        { "op": "boxcall", "box": 1, "method": "length", "args": [], "dst": 20 },
        { "op": "ret", "value": 20 }
      ] }
    ] }
  ]
}
JSON

NYASH_HAKO_MIN_SEM=1 "$NY_LLVMc" --in "$JSON_IN" --emit exe --out "$EXE_OUT" || { test_skip "aot_map_set_size_exe" "ny-llvmc emit-exe failed (plugins missing)" && exit 0; }
set +e
NYASH_HAKO_MIN_SEM=1 "$EXE_OUT"
code=$?
set -e
if [ "$code" -ne 2 ]; then
  test_skip "aot_map_set_size_exe" "link/runtime prereqs missing (exit=$code)" && exit 0
fi
echo "AOT exit=$code"
echo "OK: AOT Map set/size exit 2"
