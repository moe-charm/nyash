#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
file="apps/tests/macrobox_example.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_BOX=1
export NYASH_MACRO_BOX_ENABLE=UppercasePrintMacro

out=$(
  "$bin" "$file" 2>&1 | sed -e 's/\r$//' || true
)

echo "$out"

if ! echo "$out" | grep -q "HELLO WORLD"; then
  echo "expected HELLO WORLD in output" >&2
  exit 2
fi
if ! echo "$out" | grep -q "lower stays lower"; then
  echo "expected lower stays lower in output" >&2
  exit 3
fi

echo "[OK] MacroBox example smoke passed"

