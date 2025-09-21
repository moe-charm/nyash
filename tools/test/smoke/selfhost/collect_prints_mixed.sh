#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../../.." && pwd)

echo "[smoke] collect_prints mixed order ..." >&2

pushd "$ROOT_DIR" >/dev/null

cargo build --release -q

export NYASH_ENABLE_USING=1
export NYASH_VM_USE_PY=1
BIN=./target/release/nyash
# Use JSON Box based app to avoid reliance on MiniVmPrints fallbacks
APP=apps/tests/jsonbox_collect_prints_smoke.nyash

out=$("$BIN" --backend vm "$APP")

expected=$'A\nB\n7\n1\n7\n5'

if [[ "$out" != "$expected" ]]; then
  echo "[smoke] FAIL: unexpected output" >&2
  echo "--- got ---" >&2
  printf '%s\n' "$out" >&2
  echo "--- exp ---" >&2
  printf '%s\n' "$expected" >&2
  exit 1
fi

echo "[smoke] OK: collect_prints mixed order" >&2
popd >/dev/null
