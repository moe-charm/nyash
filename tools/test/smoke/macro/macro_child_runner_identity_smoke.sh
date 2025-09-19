#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
runner="apps/macros/expand_runner.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

json='{"kind":"Program","statements":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"x"}}}]}'

out=$("$bin" --backend vm "$runner" -- "$json" 2>&1)
echo "$out"

echo "$out" | grep -q '"value":"x"'
echo "[OK] macro_child_runner_identity passed"

