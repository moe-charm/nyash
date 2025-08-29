#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then set -x; fi

if [[ $# -lt 1 ]]; then
  echo "Usage: tools/build_python_aot.sh <input.nyash> [-o <output>]" >&2
  exit 1
fi

INPUT=""
OUT="app"
while [[ $# -gt 0 ]]; do
  case "$1" in
    -o) OUT="$2"; shift 2 ;;
    *) INPUT="$1"; shift ;;
  esac
done

if [[ ! -f "$INPUT" ]]; then
  echo "error: input not found: $INPUT" >&2
  exit 2
fi

echo "[1/3] Build Nyash (Cranelift)"
cargo build --release --features cranelift-jit >/dev/null

echo "[2/3] Build Python plugin"
(cd plugins/nyash-python-plugin && cargo build --release >/dev/null)

echo "[3/3] AOT link -> $OUT"
bash tools/build_aot.sh "$INPUT" -o "$OUT"

echo "✅ Done: $OUT"
echo "Hint: set NYASH_PY_AUTODECODE=1 at runtime for primitive returns"

