#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

# nested_if_continue: expect 1,3,5
out_c=$("$bin" --backend vm apps/tests/macro/loopform/nested_if_continue.nyash)
exp_c=$'1\n3\n5'
if [ "$(printf '%s' "$out_c" | tr -d '\r')" != "$(printf '%s' "$exp_c")" ]; then
  echo "[FAIL] nested_if_continue output mismatch" >&2
  echo "--- got ---" >&2; printf '%s\n' "$out_c" >&2
  echo "--- exp ---" >&2; printf '%s\n' "$exp_c" >&2
  exit 2
fi

# nested_if_break: expect 0,1,2
out_b=$("$bin" --backend vm apps/tests/macro/loopform/nested_if_break.nyash)
exp_b=$'0\n1\n2'
if [ "$(printf '%s' "$out_b" | tr -d '\r')" != "$(printf '%s' "$exp_b")" ]; then
  echo "[FAIL] nested_if_break output mismatch" >&2
  echo "--- got ---" >&2; printf '%s\n' "$out_b" >&2
  echo "--- exp ---" >&2; printf '%s\n' "$exp_b" >&2
  exit 3
fi

echo "[OK] loop nested-if break/continue outputs matched"
exit 0

