#!/usr/bin/env bash
set -euo pipefail

# Curated LLVM Stage-3 acceptance smoke (llvmlite harness)
# Usage: tools/smokes/curated_llvm_stage3.sh [--phi-off]

ROOT_DIR=$(cd "$(dirname "$0")/../.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if ! [ -x "$BIN" ]; then
  echo "[curated-llvm-stage3] building nyash (release, features=llvm)" >&2
  cargo build --release --features llvm >/dev/null
fi

export NYASH_LLVM_USE_HARNESS=1

if [[ "${1:-}" == "--phi-off" ]]; then
  export NYASH_MIR_NO_PHI=1
  export NYASH_VERIFY_ALLOW_NO_PHI=1
  echo "[curated-llvm-stage3] PHI-off (edge-copy) enabled" >&2
fi

run_ok() {
  local path="$1"
  echo "[curated-llvm-stage3] RUN --backend llvm: ${path}"
  timeout 10s "$BIN" --backend llvm "$path" >/dev/null
}

# A) try/catch/finally without actual throw
run_ok "$ROOT_DIR/apps/tests/stage3_try_finally_basic.nyash"

# B) throw in dead branch (acceptance only)
run_ok "$ROOT_DIR/apps/tests/stage3_throw_dead_branch.nyash"

# C) repeat with trap disabled (robustness; should be no-op here)
NYASH_LLVM_TRAP_ON_THROW=0 run_ok "$ROOT_DIR/apps/tests/stage3_try_finally_basic.nyash"
NYASH_LLVM_TRAP_ON_THROW=0 run_ok "$ROOT_DIR/apps/tests/stage3_throw_dead_branch.nyash"

echo "[curated-llvm-stage3] OK"

