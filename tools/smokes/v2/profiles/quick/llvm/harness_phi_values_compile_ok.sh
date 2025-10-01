#!/usr/bin/env bash
# Quick harness compile check: PHI with new values [{value,block}] shape

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"; while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do ROOT="$(dirname "$ROOT")"; done
NY_LLVMc="$ROOT/target/release/ny-llvmc"
OBJ_OUT="$ROOT/tmp/phi_values_ok.o"
JSON_IN="$ROOT/tmp/phi_values_ok.json"

if [ ! -x "$NY_LLVMc" ]; then (cd "$ROOT/crates/nyash-llvm-compiler" && cargo build --release); fi

cat > "$JSON_IN" << 'JSON'
{
  "version": 0,
  "functions": [
    { "name": "main", "params": [], "blocks": [
      { "id": 0, "instructions": [
        { "op": "const", "dst": 10, "value": { "type": "i64", "value": 0 } },
        { "op": "branch", "cond": 10, "then": 1, "else": 2 }
      ] },
      { "id": 1, "instructions": [
        { "op": "const", "dst": 20, "value": { "type": "i64", "value": 123 } },
        { "op": "jump", "target": 3 }
      ] },
      { "id": 2, "instructions": [
        { "op": "const", "dst": 30, "value": { "type": "i64", "value": 456 } },
        { "op": "jump", "target": 3 }
      ] },
      { "id": 3, "instructions": [
        { "op": "phi", "dst": 40, "values": [ {"value": 20, "block": 1}, {"value": 30, "block": 2} ] },
        { "op": "ret", "value": 40 }
      ] }
    ] }
  ]
}
JSON

NYASH_LLVM_PHI_STRICT=1 "$NY_LLVMc" --in "$JSON_IN" --out "$OBJ_OUT"
test -s "$OBJ_OUT"
echo "OK: harness compiled PHI(values) -> object"

