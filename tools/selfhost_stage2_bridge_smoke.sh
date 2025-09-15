#!/usr/bin/env bash
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

TMP="$ROOT_DIR/tmp"
mkdir -p "$TMP"

pass() { echo "✅ $1" >&2; }
fail() { echo "❌ $1" >&2; echo "$2" >&2; exit 1; }

compile_json() {
  local src_text="$1"
  printf "%s\n" "$src_text" > "$TMP/ny_parser_input.ny"
  # Build a local parser EXE (no pack) and run it
  "$ROOT_DIR/tools/build_compiler_exe.sh" --no-pack -o nyash_compiler_smoke >/dev/null
  "$ROOT_DIR/nyash_compiler_smoke" "$TMP/ny_parser_input.ny"
}

run_case_bridge() {
  local name="$1"; shift
  local src="$1"; shift
  local regex="$1"; shift
  set +e
  JSON=$(compile_json "$src")
  OUT=$(printf '%s\n' "$JSON" | NYASH_VM_USE_PY=1 "$BIN" --ny-parser-pipe --backend vm 2>&1)
  set -e
  if echo "$OUT" | rg -q "$regex"; then pass "$name"; else fail "$name" "$OUT"; fi
}

# A) arithmetic
run_case_bridge "arith (bridge)" 'return 1+2*3' '^Result:\s*7\b'

# B) unary minus
run_case_bridge "unary (bridge)" 'return -3 + 5' '^Result:\s*2\b'

# C) logical AND
run_case_bridge "and (bridge)" 'return (1 < 2) && (2 < 3)' '^Result:\s*true\b'

# D) ArrayBox push/size -> 2
read -r -d '' SRC_ARR <<'NY'
local a = new ArrayBox()
a.push(1)
a.push(2)
return a.size()
NY
run_case_bridge "array push/size (bridge)" "$SRC_ARR" '^Result:\s*2\b'

# E) String.length() -> 3
run_case_bridge "string length (bridge)" 'local s = "abc"; return s.length()' '^Result:\s*3\b'

echo "All selfhost Stage-2 bridge smokes PASS" >&2
exit 0
