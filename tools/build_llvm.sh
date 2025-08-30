#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  set -x
fi

usage() {
  cat << USAGE
Usage: tools/build_llvm.sh <input.nyash> [-o <output>]

Compiles a Nyash program with the LLVM backend to an object (.o),
links it with the NyRT static runtime, and produces a native executable.

Options:
  -o <output>   Output executable name (default: app)

Requirements:
  - LLVM 18 development (llvm-config-18)
  - NyRT static runtime (crates/nyrt)
USAGE
}

if [[ $# -lt 1 ]]; then usage; exit 1; fi

INPUT=""
OUT="app"
while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    -o) OUT="$2"; shift 2 ;;
    *) INPUT="$1"; shift ;;
  esac
done

if [[ ! -f "$INPUT" ]]; then
  echo "error: input file not found: $INPUT" >&2
  exit 1
fi

if ! command -v llvm-config-18 >/dev/null 2>&1; then
  echo "error: llvm-config-18 not found (install LLVM 18 dev)." >&2
  exit 2
fi

echo "[1/4] Building nyash (feature=llvm) ..."
LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm >/dev/null

echo "[2/4] Emitting object (.o) via LLVM backend ..."
OBJ="nyash_llvm_temp.o"
rm -f "$OBJ"
LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) ./target/release/nyash --backend llvm "$INPUT" >/dev/null || true
if [[ ! -f "$OBJ" ]]; then
  echo "error: object not generated: $OBJ" >&2
  exit 3
fi

echo "[3/4] Building NyRT static runtime ..."
( cd crates/nyrt && cargo build --release >/dev/null )

echo "[4/4] Linking $OUT ..."
cc "$OBJ" \
  -L crates/nyrt/target/release \
  -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
  -lpthread -ldl -lm -o "$OUT"

echo "✅ Done: $OUT"
echo "   (runtime requires nyash.toml and plugin .so per config)"

