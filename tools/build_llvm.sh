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
  -o <output>   Output executable path (default: tmp/app)

Requirements:
  - LLVM 18 development (llvm-config-18)
  - NyRT static runtime (crates/nyrt)
USAGE
}

if [[ $# -lt 1 ]]; then usage; exit 1; fi

INPUT=""
OUT="tmp/app"
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

echo "[1/4] Building nyash (feature selectable) ..."
_LLVMPREFIX=$(llvm-config-18 --prefix)
# Select LLVM feature: default harness (llvm), or legacy inkwell when NYASH_LLVM_FEATURE=llvm-inkwell-legacy
LLVM_FEATURE=${NYASH_LLVM_FEATURE:-llvm}
# Use 24 threads for parallel build
LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
  CARGO_INCREMENTAL=1 cargo build --release -j 24 -p nyash-rust --features "$LLVM_FEATURE" >/dev/null

echo "[2/4] Emitting object (.o) via LLVM backend ..."
# Default object output path under target/aot_objects
mkdir -p "$PWD/target/aot_objects"
stem=$(basename "$INPUT")
stem=${stem%.nyash}
OBJ="${NYASH_LLVM_OBJ_OUT:-$PWD/target/aot_objects/${stem}.o}"
if [[ "${NYASH_LLVM_SKIP_EMIT:-0}" != "1" ]]; then
  rm -f "$OBJ"
  COMPILER_MODE=${NYASH_LLVM_COMPILER:-harness}
  case "$COMPILER_MODE" in
    crate)
      # Use crates/nyash-llvm-compiler (ny-llvmc): requires pre-generated MIR JSON path in NYASH_LLVM_MIR_JSON
      if [[ -z "${NYASH_LLVM_MIR_JSON:-}" ]]; then
        # Auto‑emit MIR JSON via nyash CLI flag
        mkdir -p tmp
        NYASH_LLVM_MIR_JSON="tmp/nyash_crate_mir.json"
        echo "    emitting MIR JSON: $NYASH_LLVM_MIR_JSON" >&2
        ./target/release/nyash --emit-mir-json "$NYASH_LLVM_MIR_JSON" --backend mir "$INPUT" >/dev/null
      fi
      echo "    using ny-llvmc (crate) with JSON: $NYASH_LLVM_MIR_JSON" >&2
      cargo build --release -p nyash-llvm-compiler >/dev/null
      # Optional schema validation when explicitly requested
      if [[ "${NYASH_LLVM_VALIDATE_JSON:-0}" == "1" ]]; then
        if [[ -f tools/validate_mir_json.py ]]; then
          if ! python3 -m jsonschema --version >/dev/null 2>&1; then
            echo "[warn] jsonschema not available; skipping schema validation" >&2
          else
            echo "    validating MIR JSON schema ..." >&2
            python3 tools/validate_mir_json.py "$NYASH_LLVM_MIR_JSON" || {
              echo "error: MIR JSON validation failed" >&2; exit 3; }
          fi
        fi
      fi
      if [[ "${NYASH_LLVM_EMIT:-obj}" == "exe" ]]; then
        echo "    emitting EXE via ny-llvmc (crate) ..." >&2
        # Ensure NyRT is built (for libnyrt.a)
        if [[ ! -f crates/nyrt/target/release/libnyrt.a && "${NYASH_LLVM_SKIP_NYRT_BUILD:-0}" != "1" ]]; then
          ( cd crates/nyrt && cargo build --release -j 24 >/dev/null )
        fi
        NYRT_DIR_HINT="${NYASH_LLVM_NYRT:-crates/nyrt/target/release}"
        ./target/release/ny-llvmc --in "$NYASH_LLVM_MIR_JSON" --out "$OUT" --emit exe --nyrt "$NYRT_DIR_HINT" ${NYASH_LLVM_LIBS:+--libs "$NYASH_LLVM_LIBS"}
        echo "✅ Done: $OUT"; echo "   (runtime may require nyash.toml and plugins depending on app)"; exit 0
      else
        ./target/release/ny-llvmc --in "$NYASH_LLVM_MIR_JSON" --out "$OBJ"
      fi
      ;;
  esac
  if [[ "$COMPILER_MODE" == "harness" ]]; then
    if [[ "${NYASH_LLVM_FEATURE:-llvm}" == "llvm-inkwell-legacy" ]]; then
      # Legacy path: do not use harness
      NYASH_LLVM_OBJ_OUT="$OBJ" LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
        ./target/release/nyash --backend llvm "$INPUT" >/dev/null || true
    else
      # Harness path (Python llvmlite)
      NYASH_LLVM_OBJ_OUT="$OBJ" NYASH_LLVM_USE_HARNESS=1 LLVM_SYS_181_PREFIX="${_LLVMPREFIX}" LLVM_SYS_180_PREFIX="${_LLVMPREFIX}" \
        ./target/release/nyash --backend llvm "$INPUT" >/dev/null || true
    fi
  fi
fi
if [[ ! -f "$OBJ" ]]; then
  echo "error: object not generated: $OBJ" >&2
  echo "hint: you can pre-generate it (e.g. via --run-task smoke_obj_*) and set NYASH_LLVM_SKIP_EMIT=1" >&2
  exit 3
fi

if [[ "${NYASH_LLVM_ONLY_OBJ:-0}" == "1" ]]; then
  echo "[3/4] Skipping link: object generated at $OBJ (NYASH_LLVM_ONLY_OBJ=1)"
  exit 0
fi

echo "[3/4] Building NyRT static runtime ..."
if [[ "${NYASH_LLVM_SKIP_NYRT_BUILD:-0}" == "1" ]]; then
  echo "    Skipping NyRT build (NYASH_LLVM_SKIP_NYRT_BUILD=1)"
else
  # Use 24 threads for parallel build
  ( cd crates/nyrt && cargo build --release -j 24 >/dev/null )
fi

# Ensure output directory exists
mkdir -p "$(dirname "$OUT")"
echo "[4/4] Linking $OUT ..."
cc "$OBJ" \
  -L target/release \
  -L crates/nyrt/target/release \
  -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
  -lpthread -ldl -lm -o "$OUT"

echo "✅ Done: $OUT"
echo "   (runtime requires nyash.toml and plugin .so per config)"
