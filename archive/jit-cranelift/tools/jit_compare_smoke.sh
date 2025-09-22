#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT_DIR"

echo "[build] nyash (cranelift-jit)"
cargo build --release --features cranelift-jit

run_case() {
  local app="$1"
  echo "[run] jit-direct: $app"
  NYASH_JIT_THRESHOLD=1 ./target/release/nyash --jit-direct "$app"
}

run_case apps/tests/mir-branch-ret/main.nyash
run_case apps/tests/mir-compare-multi/main.nyash

echo "[ok] jit compare smokes completed"

