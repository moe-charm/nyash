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

pass_case() { echo "✅ $1" >&2; }
fail_case() { echo "❌ $1" >&2; echo "$2" >&2; exit 1; }

# Case A: arithmetic
printf 'return 1+2*3\n' > "$TMP_DIR/s2_a_arith.ny"
OUT=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/s2_a_arith.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT" | rg -q '^Result:\s*7\b' && pass_case "Stage2 arithmetic" || fail_case "Stage2 arithmetic" "$OUT"

# Case B: logical and (short-circuit)
cat > "$TMP_DIR/s2_b_and.ny" <<'NY'
return (1 < 2) && (2 < 3)
NY
OUT=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/s2_b_and.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT" | rg -q '^Result:\s*true\b' && pass_case "Stage2 logical AND" || fail_case "Stage2 logical AND" "$OUT"

# Case C: logical or (short-circuit)
cat > "$TMP_DIR/s2_c_or.ny" <<'NY'
return (1 > 2) || (2 < 3)
NY
OUT=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/s2_c_or.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT" | rg -q '^Result:\s*true\b' && pass_case "Stage2 logical OR" || fail_case "Stage2 logical OR" "$OUT"

# Case D: compare eq
cat > "$TMP_DIR/s2_d_eq.ny" <<'NY'
return (1 + 1) == 2
NY
OUT=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/s2_d_eq.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT" | rg -q '^Result:\s*true\b' && pass_case "Stage2 compare ==" || fail_case "Stage2 compare ==" "$OUT"

# Case E: nested if with locals -> 200
cat > "$TMP_DIR/s2_e_nested_if.ny" <<'NY'
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
OUT=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/s2_e_nested_if.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT" | rg -q '^Result:\s*200\b' && pass_case "Stage2 nested if" || fail_case "Stage2 nested if" "$OUT"

echo "All Stage-2 bridge smokes PASS" >&2

