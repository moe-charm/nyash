#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/tests/parser/semicolon_else_edge.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_PARSER_ALLOW_SEMICOLON=1

set +e
err=$("$bin" --backend vm "$src" 2>&1 >/dev/null)
code=$?
set -e

if [ "$code" -eq 0 ]; then
  echo "[FAIL] parser accepted forbidden '} ; else' boundary"
  exit 2
fi
echo "$err" | rg -qi 'parse error' || { echo "[FAIL] parser did not report parse error" >&2; echo "$err" >&2; exit 2; }
echo "[OK] parser semicolon else-edge smoke passed"

