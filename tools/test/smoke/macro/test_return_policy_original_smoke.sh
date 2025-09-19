#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
file="apps/tests/macro/test_runner/return_policy.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

set +e
"$bin" --run-tests --test-entry wrap --test-return original "$file" >/dev/null 2>&1
code=$?
set -e

if [ "$code" -ne 7 ]; then
  echo "expected exit code 7, got $code" >&2
  exit 2
fi

echo "[OK] test_return_policy_original passed"
