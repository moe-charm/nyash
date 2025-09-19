#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/loop_normalize_macro.nyash"

trim() { perl -pe 'chomp if eof' ; }

# with_continue: expect 1,4,9 on separate lines
out_c=$("$bin" --backend vm apps/tests/macro/loopform/with_continue.nyash)
got_c=$(printf '%s' "$out_c" | trim)
exp_c=$'1\n4\n9'
if [ "$got_c" != "$exp_c" ]; then
  echo "[FAIL] with_continue output mismatch" >&2
  echo "--- got ---" >&2; printf '%s\n' "$out_c" >&2
  echo "--- exp ---" >&2; printf '%s\n' "$exp_c" >&2
  exit 2
fi

# with_break: expect 0,1,2,3 on separate lines
out_b=$("$bin" --backend vm apps/tests/macro/loopform/with_break.nyash)
got_b=$(printf '%s' "$out_b" | trim)
exp_b=$'0\n1\n2\n3'
if [ "$got_b" != "$exp_b" ]; then
  echo "[FAIL] with_break output mismatch" >&2
  echo "--- got ---" >&2; printf '%s\n' "$out_b" >&2
  echo "--- exp ---" >&2; printf '%s\n' "$exp_b" >&2
  exit 3
fi

echo "[OK] loopform continue/break outputs matched"

