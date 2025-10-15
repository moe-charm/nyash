#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  set -x
fi

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)

APPS=(
  "apps/tests/shortcircuit_nested_selective_assign.hako"
  "apps/tests/loop_if_phi.hako"
  "apps/tests/ternary_nested.hako"
)

echo "[phi] Running curated PHI invariants parity checks (PyVM vs llvmlite)" >&2

FAIL=0
for app in "${APPS[@]}"; do
  echo "[phi] case: $app" >&2
  if ! "$ROOT_DIR/tools/pyvm_vs_llvmlite.sh" "$ROOT_DIR/$app"; then
    echo "[phi] ❌ parity failed: $app" >&2
    FAIL=1
  else
    echo "[phi] ✅ parity OK: $app" >&2
  fi
done

if [[ "$FAIL" -ne 0 ]]; then
  echo "[phi] ❌ curated PHI invariants parity has failures" >&2
  exit 1
fi
echo "[phi] ✅ all curated PHI invariants cases passed"

