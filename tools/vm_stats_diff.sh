#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "Usage: $0 <stats_before.json> <stats_after.json>" >&2
  exit 1
fi

A="$1"
B="$2"

if [ ! -f "$A" ] || [ ! -f "$B" ]; then
  echo "Input files not found" >&2
  exit 1
fi

# Extract counts objects and join keys
KEYS=$(jq -r '.counts | keys[]' "$A" "$B" | sort -u)

printf "%-14s %8s %8s %8s\n" "OP" "A" "B" "+/-"
for k in $KEYS; do
  va=$(jq -r --arg k "$k" '.counts[$k] // 0' "$A")
  vb=$(jq -r --arg k "$k" '.counts[$k] // 0' "$B")
  d=$(( vb - va ))
  printf "%-14s %8d %8d %8d\n" "$k" "$va" "$vb" "$d"
done | sort -k1,1

