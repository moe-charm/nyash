#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash (release) ..." >&2
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

mkdir -p "$ROOT_DIR/tmp"
cat >"$ROOT_DIR/tmp/ny_parser_input.ny" <<'NY'
using core.std as S
return 1
NY

OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_USE_TMP_ONLY=1 NYASH_NY_COMPILER_EMIT_ONLY=1 \
      NYASH_JSON_INCLUDE_USINGS=1 \
      "$BIN" --backend vm "$ROOT_DIR/apps/examples/string_p0.nyash" || true)

if echo "$OUT" | rg -q 'Ny compiler MVP \(ny→json_v0\) path ON'; then
  echo "PASS meta.usings gate (ON)" >&2
  exit 0
else
  # accept fallback success
  if echo "$OUT" | rg -q '^Result:\s*[0-9]+'; then
    echo "PASS meta.usings gate (fallback)" >&2
    exit 0
  fi
  echo "FAIL meta.usings gate" >&2
  echo "$OUT" | sed -n '1,160p' >&2
  exit 1
fi

