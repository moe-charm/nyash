#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../.." && pwd)

echo "[smoke] loop phi values (then-continue + per-var PHI)" >&2

pushd "$ROOT_DIR" >/dev/null

cargo build --release -q

BIN=./target/release/nyash
APP=apps/tests/loop_if_phi_continue.nyash

# Run VM (PyVM) and suppress runner result line to compare pure prints
export NYASH_VM_USE_PY=1
export NYASH_JSON_ONLY=1
out=$("$BIN" --backend vm "$APP")

expected=$'7\n1'
if [[ "$out" != "$expected" ]]; then
  echo "[smoke] FAIL: unexpected output" >&2
  echo "--- got ---" >&2
  printf '%s\n' "$out" >&2
  echo "--- exp ---" >&2
  printf '%s\n' "$expected" >&2
  exit 1
fi

echo "[smoke] OK: loop phi values correct" >&2
popd >/dev/null

