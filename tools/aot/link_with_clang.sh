#!/usr/bin/env bash
# Minimal linker wrapper for native executables (dev only)
# Usage:
#   tools/aot/link_with_clang.sh -o out_exe obj1.o [obj2.o ...] [--nyrt /path/to/libnyrt.a] [--extra "<flags>"]
#
# Notes:
# - Keeps defaults conservative; does not assume system libs unless on Linux.
# - For NyRT linkage, pass --nyrt /path/to/libnyrt.a (optional).
# - Use --extra to add platform-specific libs/flags.

set -euo pipefail

if ! command -v clang >/dev/null 2>&1; then
  echo "clang not found in PATH" >&2
  exit 2
fi

OUT=""
NYRT=""
EXTRA=""
OBJS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -o)
      OUT="$2"; shift 2;;
    --nyrt)
      NYRT="$2"; shift 2;;
    --extra)
      EXTRA="$2"; shift 2;;
    --)
      shift; break;;
    -*)
      echo "Unknown flag: $1" >&2; exit 2;;
    *)
      OBJS+=("$1"); shift;;
  esac
done

# Remaining positional args are also objects
for arg in "$@"; do
  OBJS+=("$arg")
done

if [[ -z "${OUT}" || ${#OBJS[@]} -eq 0 ]]; then
  echo "Usage: $0 -o out_exe obj1.o [obj2.o ...] [--nyrt /path/to/libnyrt.a] [--extra \"<flags>\"]" >&2
  exit 2
fi

cmd=(clang "${OBJS[@]}" -o "$OUT")

if [[ -n "$NYRT" ]]; then
  # Link NyRT as a static archive (prefer whole-archive to include entry points)
  cmd+=("-Wl,--whole-archive" "$NYRT" "-Wl,--no-whole-archive")
fi

# Minimal platform defaults
UNAME=$(uname -s || echo unknown)
case "$UNAME" in
  Linux)
    cmd+=(-ldl -lpthread -lm)
    ;;
  Darwin)
    # Usually no extra defaults required; libc is linked by default
    ;;
  MINGW*|MSYS*|CYGWIN*)
    # Leave to EXTRA; user may add -lws2_32 -lbcrypt, etc.
    ;;
esac

if [[ -n "$EXTRA" ]]; then
  # shellcheck disable=SC2206
  extra_arr=($EXTRA)
  cmd+=("${extra_arr[@]}")
fi

echo "[link] ${cmd[*]}"
"${cmd[@]}"
echo "[link] done: $OUT"

