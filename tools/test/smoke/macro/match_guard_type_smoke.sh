#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/match_guard_type_basic.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1

out=$("$bin" --dump-expanded-ast-json "$src")

# Expect: no PeekExpr remains
if echo "$out" | rg -q '"kind":"PeekExpr"'; then
  echo "[FAIL] Expanded AST still contains PeekExpr for guard-type match" >&2
  exit 2
fi

echo "[OK] match guard/type normalization smoke passed"
