#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro_golden_array_mixed.nyash"
golden="$root/tools/test/golden/macro/array_mixed.expanded.json"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_BOX_NY=1
export NYASH_MACRO_BOX_CHILD_RUNNER=0
export NYASH_MACRO_BOX_NY_PATHS="apps/macros/examples/array_prepend_zero_macro.nyash"
export NYASH_ENABLE_ARRAY_LITERAL=1
export NYASH_MACRO_BOX=1

out=$("$bin" --dump-expanded-ast-json "$src")

norm() { tr -d '\n\r\t ' <<< "$1"; }
if [ "$(norm "$out")" != "$(norm "$(cat "$golden")")" ]; then
  echo "Golden mismatch (user macro array_mixed)" >&2
  diff -u <(echo "$out") "$golden" || true
  exit 2
fi

echo "[OK] golden user macro array_mixed matched"

