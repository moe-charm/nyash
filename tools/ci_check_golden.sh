#!/usr/bin/env bash
set -euo pipefail

# Minimal golden MIR check for CI/local use

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

PAIRS=(
  "local_tests/typeop_is_as_func_poc.nyash docs/status/golden/typeop_is_as_func_poc.mir.txt"
  "local_tests/typeop_is_as_poc.nyash docs/status/golden/typeop_is_as_poc.mir.txt"
  "local_tests/extern_console_log.nyash docs/status/golden/extern_console_log.mir.txt"
  "local_tests/simple_loop_test.nyash docs/status/golden/loop_simple.mir.txt"
  "local_tests/test_vm_array_getset.nyash docs/status/golden/boxcall_array_getset.mir.txt"
)

for pair in "${PAIRS[@]}"; do
  in_file="${pair%% *}"
  golden_file="${pair##* }"
  echo "[GOLDEN] Checking: $in_file vs $golden_file"
  ./tools/compare_mir.sh "$in_file" "$golden_file"
done

echo "All golden MIR snapshots match."
