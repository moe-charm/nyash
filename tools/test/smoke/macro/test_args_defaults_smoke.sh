#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
file="apps/tests/macro/test_runner/args_defaults.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_TEST_ARGS_DEFAULTS=1

out=$("$bin" --run-tests "$file" 2>&1 | sed -e 's/\r$//')
echo "$out"

grep -q "PASS test_param_zero" <<<"$out"
grep -q "PASS test_param_pair" <<<"$out"

echo "[OK] test_args_defaults passed"
