#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

echo "[deny-direct] scanning src/llvm_py for direct vmap.get reads ..."
rg -n "vmap\\.get\\(" src/llvm_py \
  -g '!src/llvm_py/resolver.py' \
  -g '!src/llvm_py/llvm_builder.py' || true

echo "[hint] Prefer resolver.resolve_i64/resolve_ptr with (builder.block, preds, block_end_values, vmap, bb_map)."

