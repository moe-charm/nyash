#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_PARSER_STAGE3=1

src="apps/tests/macro/exception/expr_postfix_direct.nyash"
out=$("$bin" --backend vm "$root/$src" 2>/dev/null)
count=$(printf "%s" "$out" | rg -n "^cleanup$" | wc -l | tr -d ' ')
test "$count" = "2" || { echo "[FAIL] expected 2 cleanup prints, got $count" >&2; echo "$out" >&2; exit 2; }
echo "[OK] direct postfix catch/cleanup output passed"
exit 0

