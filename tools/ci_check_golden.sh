#!/usr/bin/env bash
set -euo pipefail

# Minimal golden MIR check for CI/local use

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

PAIRS=(
  "local_tests/typeop_is_as_func_poc.nyash docs/status/golden/typeop_is_as_func_poc.mir.txt"
  "local_tests/typeop_is_as_poc.nyash docs/status/golden/typeop_is_as_poc.mir.txt"
)

for pair in "${PAIRS[@]}"; do
  in_file="${pair%% *}"
  golden_file="${pair##* }"
  echo "[GOLDEN] Checking: $in_file vs $golden_file"
  ./tools/compare_mir.sh "$in_file" "$golden_file"
done

echo "All golden MIR snapshots match."

