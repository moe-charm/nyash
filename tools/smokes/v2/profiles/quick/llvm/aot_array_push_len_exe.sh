#!/usr/bin/env bash
# Quick smoke: ArrayBox push + length via AOT (expects len=3 → exit=3)

set -euo pipefail

# Resolve repo root
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"; while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do ROOT="$(dirname "$ROOT")"; done
NY_LLVMc="$ROOT/target/release/ny-llvmc"
EXE_OUT="$ROOT/tmp/aot_arr_len"
JSON_IN="$ROOT/tmp/aot_arr_len.json"

if [ ! -x "$NY_LLVMc" ]; then (cd "$ROOT/crates/nyash-llvm-compiler" && cargo build --release); fi
(cd "$ROOT/crates/hako_kernel" && cargo build --release) || true

mkdir -p "$(dirname "$EXE_OUT")"
cat > "$JSON_IN" << 'JSON'
{
  "version": 0,
  "functions": [
    { "name": "main", "params": [], "blocks": [
      { "id": 0, "instructions": [
        { "op": "mir_call", "dst": 1, "mir_call": { "callee": { "type": "Constructor", "box_type": "ArrayBox" }, "args": [] } },
        { "op": "const", "dst": 2, "value": { "type": "string", "value": "a" } },
        { "op": "const", "dst": 3, "value": { "type": "string", "value": "b" } },
        { "op": "const", "dst": 4, "value": { "type": "string", "value": "c" } },
        { "op": "boxcall", "box": 1, "method": "push", "args": [2], "dst": 10 },
        { "op": "boxcall", "box": 1, "method": "push", "args": [3], "dst": 11 },
        { "op": "boxcall", "box": 1, "method": "push", "args": [4], "dst": 12 },
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
test "$code" -eq 3
echo "OK: AOT Array push/len exit 3"
