#!/usr/bin/env bash
# Quick smoke: LLVM harness compare+branch → exe → run (expects exit=1)

set -euo pipefail

# Resolve repo root by walking up to Cargo.toml
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"
while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do
  ROOT="$(dirname "$ROOT")"
done

NY_LLVMc="$ROOT/target/release/ny-llvmc"
LIB_DIR="$ROOT/target/release"
EXE_OUT="$ROOT/tmp/aot_cmp"
JSON_IN="$ROOT/tmp/compare_branch_quick.json"

# Ensure tools exist (build if missing)
if [ ! -x "$NY_LLVMc" ]; then
  (cd "$ROOT/crates/nyash-llvm-compiler" && cargo build --release)
fi
if [ ! -f "$LIB_DIR/libhako_kernel.a" ] && [ ! -f "$LIB_DIR/libnyash_kernel.a" ]; then
  (cd "$ROOT/crates/hako_kernel" && cargo build --release) || true
fi

cat > "$JSON_IN" << 'JSON'
{
  "version": 0,
  "functions": [
    { "name": "main", "params": [], "blocks": [
      { "id": 0, "instructions": [
        { "op": "const", "dst": 1, "value": { "type": "i64", "value": 3 } },
        { "op": "const", "dst": 2, "value": { "type": "i64", "value": 5 } },
        { "op": "compare", "dst": 3, "operation": "<", "lhs": 1, "rhs": 2 },
        { "op": "branch", "cond": 3, "then": 1, "else": 2 }
      ] },
      { "id": 1, "instructions": [
        { "op": "const", "dst": 4, "value": { "type": "i64", "value": 1 } },
        { "op": "ret", "value": 4 }
      ] },
      { "id": 2, "instructions": [
        { "op": "const", "dst": 5, "value": { "type": "i64", "value": 0 } },
        { "op": "ret", "value": 5 }
      ] }
    ] }
  ]
}
JSON

"$NY_LLVMc" --in "$JSON_IN" --emit exe --out "$EXE_OUT"
set +e
"$EXE_OUT"
code=$?
set -e
echo "AOT exit=$code"
test "$code" -eq 1
echo "OK: AOT compare→branch exe exit 1"
