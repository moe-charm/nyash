#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/tests/sugar/not_basic.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_VM_USE_PY=1
export NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1
out=$("$bin" --backend vm "$src" 2>&1 | sed '/^\[entry\] Warning/d')
# Expect lines: 1 then 0
line1=$(printf '%s\n' "$out" | sed -n '1p')
line2=$(printf '%s\n' "$out" | sed -n '2p')
test "$line1" = "1" || { echo "[FAIL] not on 0 expected 1, got '$line1'" >&2; echo "$out" >&2; exit 2; }
test "$line2" = "0" || { echo "[FAIL] not on 1 expected 0, got '$line2'" >&2; echo "$out" >&2; exit 2; }
echo "[OK] not-operator smoke passed"
