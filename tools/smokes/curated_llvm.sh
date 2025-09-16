#!/usr/bin/env bash
set -euo pipefail

# Curated LLVM smoke runner (llvmlite harness)
# Usage: tools/smokes/curated_llvm.sh [--phi-off]

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

# Ensure release binary with LLVM feature
if ! [ -x "$BIN" ]; then
  echo "[curated-llvm] building nyash (release, features=llvm)" >&2
  cargo build --release --features llvm >/dev/null
fi

export NYASH_LLVM_USE_HARNESS=1

# Optional: PHI-off mode
if [[ "${1:-}" == "--phi-off" ]]; then
  export NYASH_MIR_NO_PHI=1
  export NYASH_VERIFY_ALLOW_NO_PHI=1
  echo "[curated-llvm] PHI-off (edge-copy) enabled" >&2
fi

run() {
  local path="$1"
  echo "[curated-llvm] RUN --backend llvm: ${path}"
  timeout 10s "$BIN" --backend llvm "$path" >/dev/null
}

# Core minimal (existing harness sample)
run "$ROOT_DIR/examples/llvm11_core_smoke.nyash"

# Async/await (LLVM only)
run "$ROOT_DIR/apps/tests/async-await-min/main.nyash"
run "$ROOT_DIR/apps/tests/async-spawn-instance/main.nyash"
NYASH_AWAIT_MAX_MS=100 run "$ROOT_DIR/apps/tests/async-await-timeout-fixed/main.nyash"

# Control-flow: nested loop break/continue and loop+if phi pattern
run "$ROOT_DIR/apps/tests/nested_loop_inner_break_isolated.nyash"
run "$ROOT_DIR/apps/tests/nested_loop_inner_continue_isolated.nyash"
run "$ROOT_DIR/apps/tests/loop_if_phi.nyash"

# Peek expression
run "$ROOT_DIR/apps/tests/peek_expr_block.nyash"

# Try/finally control-flow without actual throw
run "$ROOT_DIR/apps/tests/try_finally_break_inner_loop.nyash"

echo "[curated-llvm] OK"
