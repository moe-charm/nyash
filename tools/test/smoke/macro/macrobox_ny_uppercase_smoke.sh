#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
host="apps/tests/macrobox_example.nyash"
ny="apps/tests/macrobox_ny/uppercase_macro.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_BOX_NY=1
export NYASH_MACRO_BOX_NY_PATHS="$ny"

out=$("$bin" "$host" 2>&1 | sed -e 's/\r$//')
echo "$out"

grep -q "HELLO WORLD" <<<"$out"
grep -q "lower stays lower" <<<"$out"

echo "[OK] macrobox_ny_uppercase passed"

