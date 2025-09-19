#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/for_foreach_macro.nyash"
export NYASH_MACRO_BOX_CHILD=0

trim() { perl -pe 'chomp if eof' ; }

# for_
out_for=$("$bin" --backend vm apps/tests/macro/loopform/for_basic.nyash)
got_for=$(printf '%s' "$out_for" | trim)
exp_for=$'0\n1\n2'
if [ "$got_for" != "$exp_for" ]; then
  echo "[FAIL] for_ output mismatch" >&2
  echo "--- got ---" >&2; printf '%s\n' "$out_for" >&2
  echo "--- exp ---" >&2; printf '%s\n' "$exp_for" >&2
  exit 2
fi

# foreach_
out_fe=$("$bin" --backend vm apps/tests/macro/loopform/foreach_basic.nyash)
got_fe=$(printf '%s' "$out_fe" | trim)
exp_fe=$'1\n2\n3'
if [ "$got_fe" != "$exp_fe" ]; then
  echo "[FAIL] foreach_ output mismatch" >&2
  echo "--- got ---" >&2; printf '%s\n' "$out_fe" >&2
  echo "--- exp ---" >&2; printf '%s\n' "$exp_fe" >&2
  exit 3
fi

echo "[OK] for_/foreach_ output matched"
