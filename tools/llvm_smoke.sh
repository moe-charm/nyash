#!/usr/bin/env bash
set -euo pipefail

# LLVM Phase 11.2 smoke test
# - Builds nyash with LLVM backend
# - Compiles example via --backend llvm (object emission inside)
# - Verifies object file presence and non-zero size

MODE=${1:-release}
BIN=./target/${MODE}/nyash
OBJ=nyash_llvm_temp.o

if ! command -v llvm-config-18 >/dev/null 2>&1; then
  echo "error: llvm-config-18 not found. Please install LLVM 18 dev packages." >&2
  exit 2
fi

echo "[llvm-smoke] building nyash (${MODE}, feature=llvm)..." >&2
LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build -q ${MODE:+--${MODE}} --features llvm

echo "[llvm-smoke] running --backend llvm on examples/llvm11_core_smoke.nyash ..." >&2
rm -f "$OBJ"
LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) "$BIN" --backend llvm examples/llvm11_core_smoke.nyash >/dev/null || true

if [[ ! -f "$OBJ" ]]; then
  echo "error: expected object not found: $OBJ" >&2
  exit 1
fi
if [[ ! -s "$OBJ" ]]; then
  echo "error: object is empty: $OBJ" >&2
  exit 1
fi

echo "[llvm-smoke] OK: object generated: $OBJ ($(stat -c%s "$OBJ") bytes)" >&2
