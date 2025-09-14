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

# Enable me dummy injection for entire run
export NYASH_BRIDGE_ME_DUMMY=1
export NYASH_BRIDGE_ME_CLASS=ConsoleBox

# Case 1: me bound to a var and unused (ensures Var("me") resolves)
cat >"$TMP_DIR/me_dummy_bind_only.ny" <<'NY'
local x = me
return 0
NY

OUT1=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/me_dummy_bind_only.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT1" | rg -q '^Result:\s*0\b' && echo "✅ me dummy (bind only) OK" || { echo "❌ me dummy (bind only) FAILED"; echo "$OUT1"; exit 1; }

# Case 2: me used inside an if branch
cat >"$TMP_DIR/me_dummy_in_if.ny" <<'NY'
if 1 < 2 {
  local y = me
}
return 0
NY

OUT2=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP_DIR/me_dummy_in_if.ny" | "$BIN" --ny-parser-pipe || true)
echo "$OUT2" | rg -q '^Result:\s*0\b' && echo "✅ me dummy (in if) OK" || { echo "❌ me dummy (in if) FAILED"; echo "$OUT2"; exit 1; }

echo "All me-dummy smokes PASS" >&2
