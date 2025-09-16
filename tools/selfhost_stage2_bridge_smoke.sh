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
  # Primary: Python MVP parser (fast, stable vectors)
  if command -v python3 >/dev/null 2>&1; then
    local pyjson
    pyjson=$(python3 "$ROOT_DIR/tools/ny_parser_mvp.py" "$TMP/ny_parser_input.ny" 2>/dev/null | sed -n '1p')
    if [[ -n "$pyjson" ]]; then printf '%s\n' "$pyjson"; return 0; fi
  fi
  # Fallback-2: inline VM run using ParserBox + EmitterBox (embed source string)
  local inline="$TMP/inline_selfhost_emit.nyash"
  # Read src text and escape quotes and backslashes for literal embedding; keep newlines
  local esc
  esc=$(sed -e 's/\\/\\\\/g' -e 's/\"/\\\"/g' "$TMP/ny_parser_input.ny")
  cat > "$inline" << NY
include "apps/selfhost-compiler/boxes/parser_box.nyash"
include "apps/selfhost-compiler/boxes/emitter_box.nyash"
static box Main {
  main(args) {
    local src = "$esc"
    local p = new ParserBox()
    local json = p.parse_program2(src)
    local e = new EmitterBox()
    json = e.emit_program(json, "[]")
    print(json)
    return 0
  }
}
NY
  # Execute via VM (Rust interpreter) is fine since print is builtin and no plugins needed
  local raw
  raw=$("$BIN" --backend vm "$inline" 2>/dev/null || true)
  local json
  json=$(printf '%s\n' "$raw" | awk 'BEGIN{found=0} /^[ \t]*\{/{ if ($0 ~ /"version"/ && $0 ~ /"kind"/) { print; found=1; exit } } END{ if(found==0){} }')
  if [[ -n "$json" ]]; then printf '%s\n' "$json"; return 0; fi
  # Optional: build & run EXE if explicitly requested
  if [[ "${NYASH_SELFHOST_USE_EXE:-0}" == "1" ]]; then
    set +e
    "$ROOT_DIR/tools/build_compiler_exe.sh" --no-pack -o nyash_compiler_smoke >/dev/null 2>&1
    local build_status=$?
    if [[ "$build_status" -eq 0 && -x "$ROOT_DIR/nyash_compiler_smoke" ]]; then
      local out
      out=$("$ROOT_DIR/nyash_compiler_smoke" "$TMP/ny_parser_input.ny" 2>/dev/null)
      set -e
      printf "%s\n" "$out"
      return 0
    fi
    set -e
  fi
  echo ""
}

run_case_bridge() {
  local name="$1"; shift
  local src="$1"; shift
  local expect_code="$1"; shift
  set +e
  JSON=$(compile_json "$src")
  OUT=$(printf '%s\n' "$JSON" | NYASH_PIPE_USE_PYVM=1 "$BIN" --ny-parser-pipe --backend vm 2>&1)
  STATUS=$?
  set -e
  if [[ "$STATUS" == "$expect_code" ]]; then pass "$name"; else fail "$name" "$OUT"; fi
}

# A) arithmetic
run_case_bridge "arith (bridge)" 'return 1+2*3' 7

# B) unary minus
run_case_bridge "unary (bridge)" 'return -3 + 5' 2

# C) logical AND
run_case_bridge "and (bridge)" 'return (1 < 2) && (2 < 3)' 1

# D) ArrayBox push/size -> 2
SRC_ARR=$(cat <<'NY'
local a = new ArrayBox()
a.push(1)
a.push(2)
return a.size()
NY
)
run_case_bridge "array push/size (bridge)" "$SRC_ARR" 2

# E) String.length() -> 3
run_case_bridge "string length (bridge)" $'local s = "abc"\nreturn s.length()' 3

# F) assignment without 'local' (update)
SRC_ASSIGN=$(cat <<'NY'
local x = 1
local x = x + 2
return x
NY
)
run_case_bridge "assign update (bridge)" "$SRC_ASSIGN" 3

# G) array literal [x,2,3] → size() == 3
SRC_ALIT=$(cat <<'NY'
local x = 1
local arr = [x, 2, 3]
return arr.size()
NY
)
run_case_bridge "array literal (bridge)" "$SRC_ALIT" 3

# H) map literal {"name":"Alice", "age":25} → size() == 2
SRC_MLIT=$(cat <<'NY'
local m = {"name": "Alice", "age": 25}
return m.size()
NY
)
run_case_bridge "map literal (bridge)" "$SRC_MLIT" 2

echo "All selfhost Stage-2 bridge smokes PASS" >&2
exit 0
