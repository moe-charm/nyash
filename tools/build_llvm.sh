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

echo "[1/4] Building nyash (feature=llvm, harness-friendly) ..."
_LLVMPREFIX=$(llvm-config-18 --prefix)
# Build only the core package to avoid compiling workspace plugin crates
LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
  CARGO_INCREMENTAL=1 cargo build --release -p nyash-rust --features llvm >/dev/null

echo "[2/4] Emitting object (.o) via LLVM backend ..."
# Default object output path under target/aot_objects
mkdir -p "$PWD/target/aot_objects"
stem=$(basename "$INPUT")
stem=${stem%.nyash}
OBJ="${NYASH_LLVM_OBJ_OUT:-$PWD/target/aot_objects/${stem}.o}"
if [[ "${NYASH_LLVM_SKIP_EMIT:-0}" != "1" ]]; then
  rm -f "$OBJ"
  NYASH_LLVM_OBJ_OUT="$OBJ" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" ./target/release/nyash --backend llvm "$INPUT" >/dev/null || true
fi
if [[ ! -f "$OBJ" ]]; then
  echo "error: object not generated: $OBJ" >&2
  echo "hint: you can pre-generate it (e.g. via --run-task smoke_obj_*) and set NYASH_LLVM_SKIP_EMIT=1" >&2
  exit 3
fi

echo "[3/4] Building NyRT static runtime ..."
if [[ "${NYASH_LLVM_SKIP_NYRT_BUILD:-0}" == "1" ]]; then
  echo "    Skipping NyRT build (NYASH_LLVM_SKIP_NYRT_BUILD=1)"
else
  ( cd crates/nyrt && cargo build --release >/dev/null )
fi

echo "[4/4] Linking $OUT ..."
cc "$OBJ" \
  -L target/release \
  -L crates/nyrt/target/release \
  -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
  -lpthread -ldl -lm -o "$OUT"

echo "✅ Done: $OUT"
echo "   (runtime requires nyash.toml and plugin .so per config)"
