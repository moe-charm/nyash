#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/loopform/foreach_empty.nyash"
golden="$root/tools/test/golden/macro/foreach_empty.expanded.json"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

normalize_json() { python3 -c 'import sys,json; print(json.dumps(json.loads(sys.stdin.read()), sort_keys=True, separators=(",", ":")))'; }

out_raw=$("$bin" --dump-expanded-ast-json "$src")
out_norm=$(printf '%s' "$out_raw" | normalize_json)
gold_norm=$(normalize_json < "$golden")

if [ "$out_norm" != "$gold_norm" ]; then
  echo "[FAIL] foreach_empty expanded JSON mismatch" >&2
  diff -u <(echo "$out_norm") <(echo "$gold_norm") || true
  exit 2
fi

echo "[OK] golden foreach_empty expansion matched"

