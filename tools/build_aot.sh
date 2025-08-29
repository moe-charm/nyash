#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  set -x
fi

usage() {
  cat << USAGE
Usage: tools/build_aot.sh <input.nyash> [-o <output>]

Builds Nyash with Cranelift, emits an AOT object (.o) for the given program,
links it with libnyrt.a into a native executable, and prints the result path.

Options:
  -o <output>   Output executable name (default: app)

Notes:
  - Requires a Cranelift-enabled build.
  - Runtime requires nyash.toml and plugin .so files resolvable by nyash.toml paths.
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

echo "[1/4] Building nyash (Cranelift) ..."
if ! cargo build --release --features cranelift-jit >/dev/null; then
  echo "error: failed to build nyash with Cranelift (feature cranelift-jit)" >&2
  exit 1
fi

echo "[2/4] Emitting object (.o) via JIT (Strict/No-fallback) ..."
rm -rf target/aot_objects && mkdir -p target/aot_objects
NYASH_AOT_OBJECT_OUT=target/aot_objects \
NYASH_USE_PLUGIN_BUILTINS=1 \
NYASH_JIT_EXEC=1 \
NYASH_JIT_ONLY=1 \
NYASH_JIT_STRICT=1 \
NYASH_JIT_ARGS_HANDLE_ONLY=1 \
NYASH_JIT_THRESHOLD=1 \
./target/release/nyash --backend vm "$INPUT" >/dev/null || true

OBJ="target/aot_objects/main.o"
if [[ ! -f "$OBJ" ]]; then
  echo "error: object not generated: $OBJ" >&2
  echo "hint: Strict mode forbids fallback. Ensure main() is lowerable under current JIT coverage." >&2
  echo "hint: Try a simpler RO example first, or expand JIT coverage for used ops." >&2
  exit 2
fi

echo "[3/4] Building libnyrt.a ..."
if [[ ! -d crates/nyrt ]]; then
  echo "error: crates/nyrt not found. Please ensure nyrt runtime crate exists." >&2
  exit 3
fi
( cd crates/nyrt && cargo build --release >/dev/null )

echo "[4/4] Linking $OUT ..."
cc "$OBJ" \
  -L crates/nyrt/target/release \
  -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
  -lpthread -ldl -lm -o "$OUT"

echo "✅ Done: $OUT"
echo "   (runtime requires nyash.toml and plugin .so per config)"
if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  echo "info: run with NYASH_CLI_VERBOSE=1 to see detailed steps and commands"
fi
