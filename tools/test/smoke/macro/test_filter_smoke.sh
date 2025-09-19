#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
file="apps/tests/macro/test_runner/filter.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$("$bin" --run-tests --test-filter api "$file" 2>&1 | sed -e 's/\r$//')
echo "$out"

grep -q "PASS test_api_ok" <<<"$out"
if echo "$out" | grep -q "PASS test_impl_skip"; then
  echo "unexpected PASS for impl_skip (filter failed)" >&2
  exit 2
fi

echo "[OK] test_filter passed"
