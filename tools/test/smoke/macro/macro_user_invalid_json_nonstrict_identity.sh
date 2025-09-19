#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro_golden_identity.nyash"
golden="$root/tools/test/golden/macro/identity.expanded.json"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/invalid_json_macro.nyash"
export NYASH_MACRO_STRICT=0   # non-strict should fall back to identity

out=$("$bin" --dump-expanded-ast-json "$src")

# Strip whitespace for robust compare
norm() { tr -d '\n\r\t ' <<< "$1"; }

if [ "$(norm "$out")" != "$(norm "$(cat "$golden")")" ]; then
  echo "Non-strict invalid JSON should fallback to identity" >&2
  diff -u <(echo "$out") "$golden" || true
  exit 2
fi

echo "[OK] invalid JSON non-strict falls back to identity"
