#!/usr/bin/env bash
# Quick smoke: MapBox set + size via AOT (expects size=2 → exit=2)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"; while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do ROOT="$(dirname "$ROOT")"; done
NY_LLVMc="$ROOT/target/release/ny-llvmc"
EXE_OUT="$ROOT/tmp/aot_map_size"
JSON_IN="$ROOT/tmp/aot_map_size.json"

if [ ! -x "$NY_LLVMc" ]; then (cd "$ROOT/crates/nyash-llvm-compiler" && cargo build --release); fi
(cd "$ROOT/crates/hako_kernel" && cargo build --release) || true

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

NYASH_HAKO_MIN_SEM=1 "$NY_LLVMc" --in "$JSON_IN" --emit exe --out "$EXE_OUT"
set +e
NYASH_HAKO_MIN_SEM=1 "$EXE_OUT"
code=$?
set -e
echo "AOT exit=$code"
test "$code" -eq 2
echo "OK: AOT Map set/size exit 2"
