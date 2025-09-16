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

run_case_stage3() {
  local name="$1"; shift
  local src="$1"; shift
  local expect_code="$1"; shift
  local file="$TMP/selfhost_stage3_${name// /_}.nyash"
  printf "%s\n" "$src" > "$file"
  # 1) Produce JSON v0 via selfhost compiler program
  set +e
  JSON=$(NYASH_JSON_ONLY=1 "$BIN" --backend vm "$ROOT_DIR/apps/selfhost-compiler/compiler.nyash" -- --stage3 "$file" 2>/dev/null | awk 'BEGIN{found=0} /^[ \t]*\{/{ if ($0 ~ /"version"/ && $0 ~ /"kind"/) { print; found=1; exit } } END{ if(found==0){} }')
  # 2) Execute JSON v0 via Bridge (prefer PyVM harness if requested)
  OUT=$(printf '%s\n' "$JSON" | NYASH_PIPE_USE_PYVM=${NYASH_PIPE_USE_PYVM:-1} "$BIN" --ny-parser-pipe --backend vm 2>&1)
  CODE=$?
  set -e
  if [[ "$CODE" == "$expect_code" ]]; then pass "$name"; else fail "$name" "$OUT"; fi
}

# A) try/catch/finally acceptance; final return 0
run_case_stage3 "try_finally" $'try { local x = 1 } catch (Error e) { local y = 2 } finally { local z = 3 }\nreturn 0' 0

# B) break acceptance under dead branch
run_case_stage3 "break_dead" $'if false { break } else { }\nreturn 0' 0

# C) continue acceptance under dead branch
run_case_stage3 "continue_dead" $'if false { continue } else { }\nreturn 0' 0

# D) throw acceptance (degrade); final return 0
run_case_stage3 "throw_accept" $'try { throw 123 } finally { }\nreturn 0' 0

echo "All selfhost Stage-3 acceptance smokes PASS" >&2
exit 0
