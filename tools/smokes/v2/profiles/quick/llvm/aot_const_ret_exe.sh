#!/usr/bin/env bash
# Quick smoke: LLVM harness → obj → link exe (NyKernel stub) → run

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"
while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do
  ROOT="$(dirname "$ROOT")"
done
NY_LLVMc="$ROOT/target/release/ny-llvmc"
LIB_DIR="$ROOT/target/release"
EXE_OUT="$ROOT/tmp/aot_const_ret"
OBJ_OUT="$EXE_OUT.o"
JSON_IN="$ROOT/tmp/const_ret_quick.json"

# Ensure binaries exist (build if missing)
if [ ! -x "$NY_LLVMc" ]; then
  (cd "$ROOT/crates/nyash-llvm-compiler" && cargo build --release)
fi
if [ ! -f "$LIB_DIR/libhako_kernel.a" ] && [ ! -f "$LIB_DIR/libnyash_kernel.a" ]; then
  (cd "$ROOT/crates/hako_kernel" && cargo build --release) || true
fi

mkdir -p "$(dirname "$EXE_OUT")"
cat > "$JSON_IN" << 'JSON'
{
  "version": 0,
  "functions": [
    { "name": "main", "params": [], "blocks": [
      { "id": 0, "instructions": [
        { "op": "const", "dst": 1, "value": { "type": "i64", "value": 0 } },
        { "op": "ret", "value": 1 }
      ] }
    ] }
  ]
}
JSON

if [ "${SMOKES_QUICK_AOT:-0}" != "1" ]; then
  echo "[SKIP] aot_const_ret_exe (enable with SMOKES_QUICK_AOT=1)"
  exit 0
fi
NYASH_HAKO_MIN_SEM=1 "$NY_LLVMc" --in "$JSON_IN" --emit exe --out "$EXE_OUT" || exit 1
NYASH_HAKO_MIN_SEM=1 "$EXE_OUT" || exit 1
code=$?
echo "AOT exit=$code"
test "$code" -eq 0
echo "OK: AOT const→ret exe exit 0"
