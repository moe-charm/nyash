#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/../../../.." && pwd)
bin="$root/target/release/nyash"
prog="$root/apps/tests/macro/loopform/nested_block_break.nyash"

out=$("$bin" --backend vm "$prog")
# Expect lines 0,1,2 then break
expected=$'0\n1\n2'
if [ "$out" != "$expected" ]; then
  echo "[FAIL] nested_block_break output mismatch" >&2
  echo "got:" >&2
  echo "$out" >&2
  exit 2
fi
echo "[OK] nested_block_break output matched"

