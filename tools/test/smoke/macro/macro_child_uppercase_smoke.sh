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

# Prepare AST JSON input by parsing host and dumping AST via --dump-ast|--expand? Not available.
# Instead, reuse AST JSON v0 bridge indirectly is complex; feed a small crafted AST for demo.
json='{"kind":"Program","statements":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"UPPER:hello"}}}]}'

out=$(printf '%s' "$json" | "$bin" --macro-expand-child "$ny" 2>&1)
echo "$out"

echo "$out" | grep -q '"value":"HELLO"'
echo "[OK] macro_child_uppercase passed"

