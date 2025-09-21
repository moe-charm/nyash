#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
tmp1="${TMPDIR:-/tmp}/devsugar1_$$.nyash"
tmp2="${TMPDIR:-/tmp}/devsugar2_$$.nyash"
trap 'rm -f "$tmp1" "$tmp2"' EXIT

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

# 1) compound ops + ++/--
"$root/tools/dev/dev_sugar_preexpand.sh" "$root/apps/tests/dev_sugar/compound_and_inc.nyash" > "$tmp1"
export NYASH_VM_USE_PY=1
out1=$("$bin" --backend vm "$tmp1" 2>/dev/null)
# i=0 -> i++ -> 1; +=2 -> 3; *=3 -> 9; -=1 -> 8; /=2 -> 4
test "$out1" = "4" || { echo "[FAIL] dev sugar compound/inc expected 4, got '$out1'" >&2; exit 2; }

echo "[OK] dev sugar preexpand smokes passed"
