#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../../.." && pwd)

echo "[smoke] collect_empty_args_using (PyVM + using) ..." >&2

pushd "$ROOT_DIR" >/dev/null

cargo build --release -q

export NYASH_ENABLE_USING=1
export NYASH_VM_USE_PY=1
# Enable seam brace safety only for this dev smoke (default-OFF elsewhere)
export NYASH_RESOLVE_FIX_BRACES=1

BIN=./target/release/nyash
APP=apps/selfhost/vm/collect_empty_args_using_smoke.nyash

out=$("$BIN" --backend vm "$APP")

# echo() -> empty line; itoa() -> 0
expected=$'\n0'

if [[ "$out" != "$expected" ]]; then
  echo "[smoke] FAIL: unexpected output" >&2
  echo "--- got ---" >&2
  printf '%s\n' "$out" >&2
  echo "--- exp ---" >&2
  printf '%s\n' "$expected" >&2
  exit 1
fi

echo "[smoke] OK: collect_empty_args_using" >&2
popd >/dev/null
