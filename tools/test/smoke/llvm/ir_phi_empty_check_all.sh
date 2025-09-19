#!/usr/bin/env bash
set -euo pipefail

# Run empty-PHI checker across a curated set of test cases
CASES=(
  apps/tests/hello_simple_llvm.nyash
  apps/tests/loop_if_phi.nyash
  apps/tests/llvm_phi_mix.nyash
  apps/tests/llvm_phi_heavy_mix.nyash
  apps/tests/llvm_phi_try_mix.nyash
)

DIR="tools/test/smoke/llvm"

for c in "${CASES[@]}"; do
  echo "[phi-empty-check-all] -> $c"
  bash "$DIR/ir_phi_empty_check.sh" "$c"
done

echo "[phi-empty-check-all] OK: all cases passed"
