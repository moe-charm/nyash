#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
file="apps/tests/macro/test_runner/basic.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$("$bin" --run-tests "$file" 2>&1 | sed -e 's/\r$//')
echo "$out"

grep -q "PASS test_true" <<<"$out"
grep -q "PASS test_one_equals_one" <<<"$out"

echo "[OK] test_runner_basic passed"
