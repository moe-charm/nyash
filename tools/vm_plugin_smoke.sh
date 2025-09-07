#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT_DIR"

echo "[build] nyash (vm)"
cargo build --release

echo "[build] core plugins (subset)"
cargo build -p nyash-counter-plugin --release

APP="apps/tests/vm-plugin-smoke-counter/main.nyash"
echo "[run] VM plugin-first strict: $APP"
NYASH_VM_PLUGIN_STRICT=1 ./target/release/nyash --backend vm "$APP"

