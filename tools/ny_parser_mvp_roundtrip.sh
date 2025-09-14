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
TMP_SRC="$ROOT_DIR/tmp/ny_parser_input.ny"
printf 'return 1+2*3\n' > "$TMP_SRC"

# Run parser MVP (Python) to JSON v0, then pipe to bridge
JSON_OUT=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_SRC")
if [[ -z "$JSON_OUT" ]]; then
  echo "error: parser produced no JSON" >&2
  exit 1
fi
echo "$JSON_OUT" | "$BIN" --ny-parser-pipe >/tmp/ny_parser_mvp_rt.out || true

if rg -q '^Result:\s*7\b' /tmp/ny_parser_mvp_rt.out; then
  echo "✅ Ny parser MVP roundtrip OK" >&2
  exit 0
else
  echo "❌ Ny parser MVP roundtrip FAILED" >&2
  cat /tmp/ny_parser_mvp_rt.out >&2 || true
  exit 2
fi
