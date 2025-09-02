#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

APPS=(
  "$ROOT_DIR/apps/tests/async-await-min/main.nyash"
  "$ROOT_DIR/apps/tests/async-spawn-instance/main.nyash"
  "$ROOT_DIR/apps/tests/async-await-timeout-fixed/main.nyash"
)

echo "[async-smokes] Building nyash (cranelift-jit)"
cargo build --release --features cranelift-jit >/dev/null

run_vm() {
  local app="$1"
  echo "[vm] $(basename $(dirname "$app"))/$(basename "$app")"
  local envs="NYASH_AWAIT_MAX_MS=5000"
  if [[ "$app" != *"async-await-timeout-fixed"* ]]; then envs="NYASH_PLUGIN_ONLY=1 ${envs}"; fi
  timeout 10s env ${envs} "$BIN" --backend vm "$app" | sed -n 's/^Result[: ]\{0,1\}//p' | tail -n 1 || true
}

run_jit() {
  local app="$1"
  echo "[jit] $(basename $(dirname "$app"))/$(basename "$app")"
  local envs="NYASH_AWAIT_MAX_MS=5000"
  if [[ "$app" != *"async-await-timeout-fixed"* ]]; then envs="NYASH_PLUGIN_ONLY=1 ${envs}"; fi
  timeout 10s env ${envs} "$BIN" --backend cranelift "$app" | sed -n 's/^📊 Result: //p; s/^Result[: ]\{0,1\}//p' | tail -n 1 || true
}

for app in "${APPS[@]}"; do
  run_vm "$app"
  run_jit "$app"
done

echo "[async-smokes] Done"
