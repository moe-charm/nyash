#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro_golden_type_is_basic.nyash"
golden="$root/tools/test/golden/macro/type_is_basic.expanded.json"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

# Macro engine may be on or off; result is identical for this case.
export NYASH_MACRO_ENABLE=1

normalize_json() {
  python3 -c 'import sys,json; print(json.dumps(json.loads(sys.stdin.read()), sort_keys=True, separators=(",",":")))'
}

out_raw=$("$bin" --dump-expanded-ast-json "$src")
out_norm=$(printf '%s' "$out_raw" | normalize_json)
gold_norm=$(normalize_json < "$golden")

if [ "$out_norm" != "$gold_norm" ]; then
  echo "Golden mismatch (type_is basic)" >&2
  diff -u <(echo "$out_norm") <(echo "$gold_norm") || true
  exit 2
fi

echo "[OK] golden type_is basic matched"

