#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash (release) ..." >&2
  cargo build --release >/dev/null
fi

TMP_DIR="$ROOT_DIR/tmp"
mkdir -p "$TMP_DIR"

# If/Else PHI merge
cat >"$TMP_DIR/phi_if_sample.ny" <<'NY'
local x = 1
if 1 < 2 {
  local x = 10
} else {
  local x = 20
}
return x
NY

OUT1=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/phi_if_sample.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT1" | rg -q '^Result:\s*10\b' && echo "✅ If/Else PHI merge OK" || { echo "❌ If/Else PHI merge FAILED"; echo "$OUT1"; exit 1; }

# Loop PHI merge
cat >"$TMP_DIR/phi_loop_sample.ny" <<'NY'
local i = 0
local s = 0
loop(i < 3) {
  local s = s + 1
  local i = i + 1
}
return s
NY

OUT2=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/phi_loop_sample.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT2" | rg -q '^Result:\s*3\b' && echo "✅ Loop PHI merge OK" || { echo "❌ Loop PHI merge FAILED"; echo "$OUT2"; exit 1; }

# If (no-else) PHI merge with base
cat >"$TMP_DIR/phi_if_noelse_sample.ny" <<'NY'
local x = 1
if 1 > 2 {
  local x = 10
}
return x
NY

OUT3=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/phi_if_noelse_sample.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT3" | rg -q '^Result:\s*1\b' && echo "✅ If(no-else) PHI merge OK" || { echo "❌ If(no-else) PHI merge FAILED"; echo "$OUT3"; exit 1; }

# Nested If PHI merge
cat >"$TMP_DIR/phi_if_nested_sample.ny" <<'NY'
local x = 1
if 1 < 2 {
  if 2 < 1 {
    local x = 100
  } else {
    local x = 200
  }
} else {
  local x = 300
}
return x
NY

OUT4=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/phi_if_nested_sample.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT4" | rg -q '^Result:\s*200\b' && echo "✅ Nested If PHI merge OK" || { echo "❌ Nested If PHI merge FAILED"; echo "$OUT4"; exit 1; }

echo "All Stage-2 PHI smokes PASS" >&2
