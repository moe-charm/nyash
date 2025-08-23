#!/usr/bin/env bash
set -euo pipefail

# Run Nyash VM with stats enabled and save JSON output
# Usage: tools/run_vm_stats.sh <nyash_file> [output_json]

if [ $# -lt 1 ]; then
  echo "Usage: $0 <nyash_file> [output_json]" >&2
  exit 1
fi

NYASH_FILE="$1"
OUT_JSON="${2:-vm_stats.json}"

if [ ! -f "$NYASH_FILE" ]; then
  echo "File not found: $NYASH_FILE" >&2
  exit 1
fi

NYASH_BIN="./target/release/nyash"
if [ ! -x "$NYASH_BIN" ]; then
  echo "Building nyash in release mode..." >&2
  cargo build --release -q
fi

echo "Running: $NYASH_BIN --backend vm --vm-stats --vm-stats-json $NYASH_FILE" >&2
NYASH_VM_STATS=1 NYASH_VM_STATS_JSON=1 "$NYASH_BIN" --backend vm --vm-stats --vm-stats-json "$NYASH_FILE" > "$OUT_JSON"
echo "Stats written to: $OUT_JSON" >&2

