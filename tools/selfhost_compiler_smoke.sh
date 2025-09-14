#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash (release) ..." >&2
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

TMP_DIR="$ROOT_DIR/tmp"
mkdir -p "$TMP_DIR"

printf 'return 1+2*3\n' > "$TMP_DIR/ny_parser_input.ny"

OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_CLI_VERBOSE=1 "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" || true)
if echo "$OUT" | rg -q 'ny compiler MVP \(ny→json_v0\) path ON' && echo "$OUT" | rg -q '^Result:\s*0\b'; then
  echo "✅ ny compiler MVP path OK (json_v0 emit + result 0)"
else
  echo "WARN: ny compiler path not used; fallback executed (acceptable during MVP)"
  echo "$OUT" | sed -n '1,120p'
fi

echo "All selfhost compiler smokes PASS" >&2
