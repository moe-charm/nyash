#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
host="apps/tests/macro_test_runner_basic.nyash"
ny="apps/tests/macrobox_ny/identity_macro.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_BOX_NY=1
export NYASH_MACRO_BOX_NY_PATHS="$ny"
export NYASH_MACRO_TRACE=1

out=$("$bin" --run-tests "$host" 2>&1 | sed -e 's/\r$//')
echo "$out"

grep -q "registered Ny MacroBox 'MacroBoxSpec'" <<<"$out"
echo "[OK] macrobox_ny_loader passed"

