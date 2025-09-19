#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/strings/upper_string.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

# Enable user macro (upper string) and macro engine
export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/upper_string_macro.nyash"

# Selfhost pre-expand: default auto (no explicit env); requires PyVM
export NYASH_USE_NY_COMPILER=1
export NYASH_VM_USE_PY=1

# Verbose to assert pre-expand path engagement
export NYASH_CLI_VERBOSE=1

out=$("$bin" --backend vm "$src" 2>&1 || true)

echo "$out" | rg -q "selfhost macro pre-expand: engaging" && echo "[OK] selfhost pre-expand (auto) engaged" && exit 0

echo "[WARN] selfhost pre-expand auto did not engage; printing logs:" >&2
echo "$out" >&2
exit 2
