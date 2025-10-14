#!/usr/bin/env bash
# Cross-compile LLVM IR (.ll) to Windows COFF .obj via clang
# Usage: tools/aot/windows/ll_to_obj.sh <in.ll> <out.obj> [--target x86_64-pc-windows-msvc]

set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <in.ll> <out.obj> [--target x86_64-pc-windows-msvc]" >&2
  exit 2
fi

IN="$1"; OUT="$2"; shift 2
TARGET="x86_64-pc-windows-msvc"
if [ "${1:-}" = "--target" ] && [ -n "${2:-}" ]; then
  TARGET="$2"; shift 2
fi

clang --target="$TARGET" -c "$IN" -o "$OUT"
echo "[ll->obj] Wrote $OUT (target=$TARGET)"

