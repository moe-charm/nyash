#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT_DIR/.."

APPS=(
  apps/tests/ternary_nested.hako
  apps/tests/loop_if_phi.hako
  apps/tests/peek_expr_block.hako
  apps/tests/string_ops_basic.hako
  apps/tests/shortcircuit_or_phi_skip.hako
)

STRICT=${CMP_STRICT:-0}

ok=0; ng=0
for app in "${APPS[@]}"; do
  echo "[parity] $app" >&2
  if CMP_STRICT=$STRICT ./tools/pyvm_vs_llvmlite.sh "$app" >/dev/null; then
    echo "[parity] OK: $app" >&2
    ok=$((ok+1))
  else
    echo "[parity] FAIL: $app" >&2
    ng=$((ng+1))
  fi
done

echo "[parity] summary: OK=$ok NG=$ng" >&2
if [[ $ng -gt 0 ]]; then
  exit 1
fi
exit 0

