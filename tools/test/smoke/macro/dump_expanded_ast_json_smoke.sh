#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
file="apps/tests/ternary_basic.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$("$bin" --dump-expanded-ast-json "$file" 2>&1)
echo "$out" | head -n 1

echo "$out" | grep -q '"kind"' || { echo "no kind in JSON" >&2; exit 2; }
echo "[OK] dump_expanded_ast_json passed"

