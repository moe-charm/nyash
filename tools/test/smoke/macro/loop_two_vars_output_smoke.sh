#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/loopform/two_vars.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/loop_normalize_macro.nyash"

out=$("$bin" --backend vm "$src")

# Normalize: strip trailing newline for comparison
trim() { perl -pe 'chomp if eof' ; }

got_norm=$(printf '%s' "$out" | trim)
expected_norm=$'0\n1\n2'

if [ "$got_norm" != "$expected_norm" ]; then
  echo "[FAIL] loop_two_vars output mismatch" >&2
  echo "--- got ---" >&2
  printf '%s' "$out" >&2
  echo "--- exp ---" >&2
  printf '%s\n' "$expected_norm" >&2
  exit 2
fi

echo "[OK] loop_two_vars output matched"
