#!/bin/bash
# wasm_placeholder.sh — Gated placeholder for WASM quick smokes

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Gate: enable only when explicitly requested
if [[ "${SMOKES_ENABLE_WASM:-0}" != "1" && "${NYASH_WASM_USE:-0}" != "1" ]]; then
  test_skip "WASM quick smokes are gated; set SMOKES_ENABLE_WASM=1 or NYASH_WASM_USE=1"
  exit 0
fi

test_wasm_placeholder() {
  # Placeholder: succeed fast; real WASM harness tests will be added here.
  echo "[wasm] placeholder PASS"
  return 0
}

run_test "wasm_placeholder" test_wasm_placeholder || exit 1
exit 0

