#!/usr/bin/env bash
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
BIN="$ROOT/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT" && cargo build --release >/dev/null)
fi

fail() { echo "❌ $1" >&2; echo "$2" >&2; exit 1; }
pass() { echo "✅ $1" >&2; }

run_json_case() {
  local name="$1"; shift
  local json_path="$1"; shift
  local expect_code="$1"; shift
  set +e
  OUT=$(NYASH_TRY_RESULT_MODE=1 NYASH_PIPE_USE_PYVM=${NYASH_PIPE_USE_PYVM:-1} \
        "$BIN" --ny-parser-pipe --backend vm < "$json_path" 2>&1)
  CODE=$?
  set -e
  if [[ "$CODE" == "$expect_code" ]]; then pass "$name"; else fail "$name (code=$CODE expected=$expect_code)" "$OUT"; fi
}

run_json_case "try_basic" "$ROOT/tests/json_v0_stage3/try_basic.json" 12
run_json_case "try_nested_if" "$ROOT/tests/json_v0_stage3/try_nested_if.json" 103
run_json_case "block_postfix_catch" "$ROOT/tests/json_v0_stage3/block_postfix_catch.json" 43
run_json_case "try_unified_hard" "$ROOT/tests/json_v0_stage3/try_unified_hard.json" 40

echo "OK: bridge try_result_mode smoke"
