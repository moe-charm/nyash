#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  set -x
fi

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")"/.. && pwd)
cd "$ROOT_DIR"

BIN=./target/release/nyash
NYLL=./target/release/ny-llvmc

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash ..." >&2
  cargo build --release >/dev/null
fi
if [[ ! -x "$NYLL" ]]; then
  echo "[build] ny-llvmc ..." >&2
  cargo build --release -p nyash-llvm-compiler >/dev/null
fi

APP="${1:-apps/tests/ternary_basic.nyash}"
OUT="${2:-tmp/crate_exe_smoke}"

mkdir -p tmp
JSON=tmp/crate_exe_smoke.json

echo "[1/3] Emitting MIR JSON via CLI ..." >&2
"$BIN" --emit-mir-json "$JSON" --backend mir "$APP" >/dev/null

echo "[2/3] Building EXE via ny-llvmc (crate) ..." >&2
NYASH_LLVM_NYRT_DIR=${NYASH_LLVM_NYRT:-target/release}
if [[ ! -f "$NYASH_LLVM_NYRT_DIR/libnyrt.a" ]]; then
  ( cd crates/nyrt && cargo build --release >/dev/null )
fi
"$NYLL" --in "$JSON" --out "$OUT" --emit exe --nyrt "$NYASH_LLVM_NYRT_DIR" ${NYASH_LLVM_LIBS:+--libs "$NYASH_LLVM_LIBS"}

echo "[3/3] Running EXE ..." >&2
set +e
"$OUT" >/dev/null 2>&1
CODE=$?
set -e
echo "exit=$CODE"

echo "✅ crate_exe_smoke OK: $OUT (exit=$CODE)"
exit 0
