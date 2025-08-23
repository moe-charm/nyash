#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: $0 <input.nyash> [output.txt]" >&2
  echo "Dumps Builder-only MIR (--no-optimize) for reproducible snapshots." >&2
  exit 2
fi

INPUT="$1"
OUTFILE="${2:-}"

if [ ! -f "$INPUT" ]; then
  echo "Input not found: $INPUT" >&2
  exit 1
fi

BIN="${NYASH_BIN:-./target/release/nyash}"
if [ ! -x "$BIN" ]; then
  echo "nyash binary not found at $BIN. Build first: cargo build --release" >&2
  exit 1
fi

CMD=("$BIN" --dump-mir --mir-verbose --no-optimize "$INPUT")

if [ -n "${NYASH_MIR_VERBOSE_EFFECTS:-}" ]; then
  CMD=("$BIN" --dump-mir --mir-verbose --mir-verbose-effects --no-optimize "$INPUT")
fi

if [ -n "$OUTFILE" ]; then
  mkdir -p "$(dirname "$OUTFILE")"
  "${CMD[@]}" | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}' > "$OUTFILE"
  echo "Wrote MIR snapshot: $OUTFILE"
else
  "${CMD[@]}"
fi
