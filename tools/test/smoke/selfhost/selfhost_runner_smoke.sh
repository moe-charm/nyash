#!/usr/bin/env bash
set -euo pipefail

# Smoke: Stage 0/1 selfhost runner wiring (harness JSON path)

ROOT_DIR=$(cd "$(dirname "$0")/../../.." && pwd)
cd "$ROOT_DIR"

echo "[smoke] building nyash (release)"
cargo build --release -q

BIN=./target/release/nyash
APP=apps/tests/dev_prints_count_probe.nyash

if [[ ! -x "$BIN" ]]; then
  echo "nyash binary not found: $BIN" >&2
  exit 1
fi

echo "[smoke] running selfhost runner (harness)"
set +e
OUT=$(NYASH_VM_USE_PY=1 NYASH_SELFHOST_EXEC=1 "$BIN" --backend vm "$APP" -- --trace 2>&1)
CODE=$?
set -e

echo "$OUT" | sed -e 's/^/[child] /'

if [[ $CODE -ne 0 ]]; then
  echo "❌ selfhost runner exited with code $CODE" >&2
  exit $CODE
fi

if ! echo "$OUT" | grep -q "\[selfhost\] harness summary: functions="; then
  echo "❌ expected harness summary line not found" >&2
  exit 1
fi

echo "✅ selfhost runner smoke passed"

