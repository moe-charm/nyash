#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro_golden_map_insert_tag.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/map_insert_tag_macro.nyash"

export NYASH_USE_NY_COMPILER=1
export NYASH_VM_USE_PY=1
export NYASH_CLI_VERBOSE=1

out=$("$bin" --backend vm "$src" 2>&1 || true)

echo "$out" | rg -q "selfhost macro pre-expand: engaging" && echo "[OK] map pre-expand (auto) engaged" && exit 0

echo "[WARN] map pre-expand auto did not engage; printing logs:" >&2
echo "$out" >&2
exit 2

