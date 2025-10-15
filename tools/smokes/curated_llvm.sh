#!/usr/bin/env bash
set -euo pipefail

# Curated LLVM smoke runner (llvmlite harness)
if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then set -x; fi

# Usage:
#   tools/smokes/curated_llvm.sh [--phi-off] [--with-if-merge] [--with-loop-prepass]
# Notes:
#   - Default is PHI-on (MIR14) with harness on.
#   - `--phi-off` switches to the legacy edge-copy mode.
#   - Flags are independent and can be combined.

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

# Ensure release binary with LLVM feature
if ! [ -x "$BIN" ]; then
  echo "[curated-llvm] building nyash (release, features=llvm)" >&2
  cargo build --release --features llvm >/dev/null
fi

export NYASH_LLVM_USE_HARNESS=1

# Defaults
export NYASH_MIR_NO_PHI=${NYASH_MIR_NO_PHI:-0}
export NYASH_VERIFY_ALLOW_NO_PHI=${NYASH_VERIFY_ALLOW_NO_PHI:-0}
unset NYASH_LLVM_PREPASS_IFMERGE || true
unset NYASH_LLVM_PREPASS_LOOP || true

WITH_IFMERGE=0
WITH_LOOP=0

# Parse flags
for arg in "$@"; do
  case "$arg" in
    --phi-off)
      export NYASH_MIR_NO_PHI=1
      export NYASH_VERIFY_ALLOW_NO_PHI=1
      echo "[curated-llvm] PHI-off (edge-copy legacy) enabled" >&2
      ;;
    --phi-on)
      export NYASH_MIR_NO_PHI=0
      export NYASH_VERIFY_ALLOW_NO_PHI=0
      echo "[curated-llvm] PHI-on (SSA builder) enforced" >&2
      ;;
    --with-if-merge)
      WITH_IFMERGE=1
      export NYASH_LLVM_PREPASS_IFMERGE=1
      echo "[curated-llvm] if-merge prepass enabled" >&2
      ;;
    --with-loop-prepass)
      WITH_LOOP=1
      export NYASH_LLVM_PREPASS_LOOP=1
      echo "[curated-llvm] loop prepass enabled" >&2
      ;;
    -h|--help)
      echo "Usage: $0 [--phi-off] [--with-if-merge] [--with-loop-prepass]"; exit 0 ;;
  esac
done

if [[ "${NYASH_MIR_NO_PHI}" == "0" ]]; then
  echo "[curated-llvm] PHI-on (SSA builder) running" >&2
else
  echo "[curated-llvm] PHI-off (edge-copy legacy) active" >&2
fi

run() {
  local path="$1"
  echo "[curated-llvm] RUN --backend llvm: ${path}"
  timeout 10s "$BIN" --backend llvm "$path" >/dev/null
}

# Core minimal (existing harness sample)
run "$ROOT_DIR/examples/llvm11_core_smoke.hako"

# Async/await (LLVM only)
run "$ROOT_DIR/apps/tests/async-await-min/main.hako"
run "$ROOT_DIR/apps/tests/async-spawn-instance/main.hako"
NYASH_AWAIT_MAX_MS=100 run "$ROOT_DIR/apps/tests/async-await-timeout-fixed/main.hako"

# Control-flow: nested loop break/continue and loop+if phi pattern
run "$ROOT_DIR/apps/tests/nested_loop_inner_break_isolated.hako"
run "$ROOT_DIR/apps/tests/nested_loop_inner_continue_isolated.hako"
run "$ROOT_DIR/apps/tests/loop_if_phi.hako"

# Peek expression
run "$ROOT_DIR/apps/tests/peek_expr_block.hako"

# Try/cleanup control-flow without actual throw
run "$ROOT_DIR/apps/tests/try_finally_break_inner_loop.hako"

# Optional: if-merge (ret-merge) tests
if [[ "$WITH_IFMERGE" == "1" ]]; then
  run "$ROOT_DIR/apps/tests/ternary_basic.hako"
  run "$ROOT_DIR/apps/tests/ternary_nested.hako"
fi

echo "[curated-llvm] OK"
