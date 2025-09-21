#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../../.." && pwd)

echo "[smoke] jsonbox nested ..." >&2
pushd "$ROOT_DIR" >/dev/null

cargo build --release -q --manifest-path plugins/nyash-json-plugin/Cargo.toml

export NYASH_VM_USE_PY=1
export NYASH_LOAD_NY_PLUGINS=1

BIN=./target/release/nyash
APP=apps/tests/jsonbox_nested.nyash

out=$("$BIN" --backend vm "$APP")
expected=$'B\n7\n3\n2'

if [[ "$out" != "$expected" ]]; then
  echo "[smoke] FAIL: unexpected output" >&2
  echo "--- got ---" >&2
  printf '%s\n' "$out" >&2
  echo "--- exp ---" >&2
  printf '%s\n' "$expected" >&2
  exit 1
fi

echo "[smoke] OK: jsonbox nested" >&2
popd >/dev/null

