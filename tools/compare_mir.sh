#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <input.nyash> <golden.mir.txt>" >&2
  exit 2
fi

INPUT="$1"
GOLDEN="$2"

TMPDIR="${TMPDIR:-/tmp}"
OUT="$TMPDIR/mir_snapshot_$$.txt"
trap 'rm -f "$OUT"' EXIT

# Allow effect annotation opt-in via env var
if [ -n "${NYASH_MIR_VERBOSE_EFFECTS:-}" ]; then
  NYASH_MIR_VERBOSE_EFFECTS=1 ./tools/snapshot_mir.sh "$INPUT" "$OUT" >/dev/null
else
  ./tools/snapshot_mir.sh "$INPUT" "$OUT" >/dev/null
fi

if ! diff -u "$GOLDEN" "$OUT"; then
  echo "MIR snapshot differs from golden: $GOLDEN" >&2
  exit 1
fi

echo "MIR matches golden: $GOLDEN"

