#!/usr/bin/env bash
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

VEC_DIR="$ROOT_DIR/tests/json_v0"
[[ -d "$VEC_DIR" ]] || { echo "No vectors at $VEC_DIR" >&2; exit 1; }

pass() { echo "✅ $1" >&2; }
fail() { echo "❌ $1" >&2; echo "$2" >&2; exit 1; }

run_vec() {
  local base="$1"; shift
  local expect_code="$1"; shift
  local path="$VEC_DIR/$base.json"
  [[ -f "$path" ]] || fail "$base (missing)" "Vector not found: $path"
  set +e
  OUT=$(cat "$path" | NYASH_PIPE_USE_PYVM=1 "$BIN" --ny-parser-pipe --backend vm 2>&1)
  STATUS=$?
  set -e
  if [[ "$STATUS" == "$expect_code" ]]; then pass "$base"; else fail "$base" "$OUT"; fi
}

# Vectors: base name -> expected Result
run_vec arith 7
run_vec if_then_else 10
run_vec while_sum 3
run_vec logical_shortcircuit_and 0
run_vec logical_shortcircuit_or 0
run_vec method_string_length 3
run_vec logical_nested 1
run_vec string_chain 2

echo "All JSON v0 vectors PASS" >&2
exit 0
