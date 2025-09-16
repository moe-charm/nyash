#!/usr/bin/env bash
# Archived: async spawn VM/JIT runs (LLVM is preferred for async)
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../" && pwd)
BIN="$ROOT_DIR/target/release/nyash"
APP="$ROOT_DIR/apps/tests/async-spawn-instance/main.nyash"

echo "[smoke] Building nyash (cranelift-jit)"
cargo build --release --features cranelift-jit >/dev/null

echo "[smoke] VM run (10s timeout)"
timeout 10s env NYASH_PLUGIN_ONLY=1 NYASH_AWAIT_MAX_MS=5000 "$BIN" --backend vm "$APP" | tee /tmp/ny_vm.out || true

echo "[smoke] JIT run (10s timeout)"
timeout 10s env NYASH_PLUGIN_ONLY=1 NYASH_AWAIT_MAX_MS=5000 "$BIN" --backend cranelift "$APP" | tee /tmp/ny_jit.out || true

echo "[smoke] LLVM AOT skipped for this test (no 'env' binding in source)"

echo "[smoke] Done"

