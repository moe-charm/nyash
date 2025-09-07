#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT_DIR"

APP="apps/tests/vm-plugin-smoke-counter/main.nyash"

echo "[build] core + counter plugin"
cargo build --release --features cranelift-jit
cargo build -p nyash-counter-plugin --release

echo "[AOT] emit + link + run"
bash tools/build_aot.sh "$APP" app_counter_aot

