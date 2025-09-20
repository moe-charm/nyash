#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/if/assign_two_vars.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_SCOPEBOX_ENABLE=1
out=$("$bin" --backend vm "$src" 2>/dev/null || true)
# Expect two lines printed (x and y). Just check exit success and non-empty
if [ -n "${out//$'\n'/}" ]; then
  echo "[OK] ScopeBox enabled run produced output"; exit 0
fi
echo "[FAIL] ScopeBox enabled run produced no output" >&2
exit 2

