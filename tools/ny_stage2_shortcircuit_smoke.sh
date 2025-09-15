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

pass() { echo "✅ $1" >&2; }
fail() { echo "❌ $1" >&2; echo "$2" | sed -n '1,160p' >&2; exit 1; }

run_bridge() {
  # Use Stage-2 Python MVP parser → JSON v0 → bridge pipe
  local src="$1"
  local json
  printf '%s\n' "$src" > "$TMP_DIR/stage2_tmp.ny"
  python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/stage2_tmp.ny" | "$BIN" --ny-parser-pipe 2>&1 || true
}

# 1) AND: LHS false → RHS not evaluated
SRC=$'local c = new ConsoleBox()\nreturn (1>2) && (c.println("rhs") == 0)'
OUT=$(run_bridge "$SRC")
echo "$OUT" | rg -q '^Result:\s*false\b' \
  && ! echo "$OUT" | rg -q '^rhs$' \
  && pass "shortcircuit: AND skips RHS" || fail "shortcircuit: AND skips RHS" "$OUT"

# 2) OR: LHS true → RHS not evaluated
SRC=$'local c = new ConsoleBox()\nreturn (1<2) || (c.println("rhs") == 0)'
OUT=$(run_bridge "$SRC")
echo "$OUT" | rg -q '^Result:\s*true\b' \
  && ! echo "$OUT" | rg -q '^rhs$' \
  && pass "shortcircuit: OR skips RHS" || fail "shortcircuit: OR skips RHS" "$OUT"

echo "All Stage-2 short-circuit (skip RHS) smokes PASS" >&2

# Nested short-circuit (no side effects) via pipe→PyVM
SRC=$'return (1 < 2) && ((1 > 2) || (2 < 3))'
printf '%s\n' "$SRC" > "$TMP_DIR/sc_nested_tmp.ny"
set +e
python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/sc_nested_tmp.ny" | NYASH_PIPE_USE_PYVM=1 "$BIN" --ny-parser-pipe >/dev/null 2>&1
CODE=$?
set -e
[[ "$CODE" -eq 1 ]] && pass "shortcircuit: nested AND/OR (pipe→pyvm)" || fail "shortcircuit: nested" "__EXIT_CODE__=$CODE"
