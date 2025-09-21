#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../../.." && pwd)

echo "[smoke] using edge cases (alias + path mix) ..." >&2

pushd "$ROOT_DIR" >/dev/null

cargo build --release -q

export NYASH_ENABLE_USING=1
export NYASH_VM_USE_PY=1
BIN=./target/release/nyash
APP=apps/tests/using_edge_mix.nyash

out=$("$BIN" --backend vm "$APP")

expected=$'1\n1'

if [[ "$out" != "$expected" ]]; then
  echo "[smoke] FAIL: unexpected output" >&2
  echo "--- got ---" >&2
  printf '%s\n' "$out" >&2
  echo "--- exp ---" >&2
  printf '%s\n' "$expected" >&2
  exit 1
fi

echo "[smoke] OK: using edge cases" >&2
popd >/dev/null
