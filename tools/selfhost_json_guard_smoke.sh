#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash (release) ..." >&2
  cargo build --release >/dev/null
fi

mkdir -p "$ROOT_DIR/tmp"
printf 'return 1+2*3\n' > "$ROOT_DIR/tmp/ny_parser_input.ny"

# Force legacy/new selection if needed by environment
OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_USE_TMP_ONLY=1 NYASH_NY_COMPILER_EMIT_ONLY=1 \
      "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" || true)

if echo "$OUT" | rg -q 'Ny compiler MVP \(ny→json_v0\) path ON'; then
  echo "✅ selfhost JSON guard OK (path ON)" >&2
  exit 0
else
  echo "❌ selfhost JSON guard FAILED" >&2
  echo "$OUT" | sed -n '1,120p' >&2
  exit 1
fi

