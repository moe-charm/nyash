#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/match/literal_basic.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$("$bin" --backend vm "$root/$src" 2>/dev/null)
test "$out" = "20" || { echo "[FAIL] expected 20, got '$out'" >&2; exit 2; }
echo "[OK] match literal_basic output passed"
exit 0

