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

compile_json_stage3() {
  local src_text="$1"
  local inline="$TMP/inline_selfhost_emit_stage3.nyash"
  # Embed source (escape quotes and backslashes; preserve newlines)
  local esc
  esc=$(printf '%s' "$src_text" | sed -e 's/\\/\\\\/g' -e 's/\"/\\\"/g')
  cat > "$inline" << NY
include "apps/selfhost-compiler/boxes/parser_box.nyash"
include "apps/selfhost-compiler/boxes/emitter_box.nyash"
static box Main {
  main(args) {
    local source_text = "$esc"
    local p = new ParserBox()
    local json = p.parse_program2(source_text)
    local e = new EmitterBox()
    json = e.emit_program(json, "[]")
    print(json)
    return 0
  }
}
NY
  local raw
  raw=$("$BIN" --backend vm "$inline" 2>/dev/null || true)
  # Extract the first JSON-looking line (contains version/kind)
  printf '%s\n' "$raw" | awk 'BEGIN{found=0} /^[ \t]*\{/{ if ($0 ~ /"version"/ && $0 ~ /"kind"/) { print; found=1; exit } } END{ if(found==0){} }'
}

run_case_stage3() {
  local name="$1"; shift
  local src="$1"; shift
  local expect_code="$1"; shift
  set +e
  JSON=$(compile_json_stage3 "$src")
  OUT=$(printf '%s\n' "$JSON" | NYASH_PIPE_USE_PYVM=1 "$BIN" --ny-parser-pipe --backend vm 2>&1)
  CODE=$?
  set -e
  if [[ "$CODE" == "$expect_code" ]]; then pass "$name"; else fail "$name" "$OUT"; fi
}

# A) try/catch/finally acceptance; final return 0
run_case_stage3 "try/catch/finally (accept)" $'try { local x = 1 } catch (Error e) { local y = 2 } finally { local z = 3 }\nreturn 0' 0

# B) break acceptance under dead branch
run_case_stage3 "break in dead branch (accept)" $'if false { break } else { }\nreturn 0' 0

# C) continue acceptance under dead branch
run_case_stage3 "continue in dead branch (accept)" $'if false { continue } else { }\nreturn 0' 0

# D) throw acceptance (degrade); final return 0
run_case_stage3 "throw (accept)" $'try { throw 123 } finally { }\nreturn 0' 0

echo "All selfhost Stage-3 acceptance smokes PASS" >&2
exit 0
