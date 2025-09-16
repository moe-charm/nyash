#!/usr/bin/env bash
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

pass() { echo "✅ $1" >&2; }
fail() { echo "❌ $1" >&2; echo "$2" >&2; exit 1; }

run_json_expect_code() {
  local name="$1"; shift
  local json="$1"; shift
  local expect_code="$1"; shift
  set +e
  # Feed JSON v0 directly via pipe. Prefer PyVM for parity.
  OUT=$(printf '%s\n' "$json" | NYASH_PIPE_USE_PYVM=${NYASH_PIPE_USE_PYVM:-1} "$BIN" --ny-parser-pipe --backend vm 2>&1)
  CODE=$?
  set -e
  if [[ "$CODE" == "$expect_code" ]]; then pass "$name"; else fail "$name" "$OUT"; fi
}

# A) try/catch/finally acceptance (degrade path unless NYASH_BRIDGE_TRY_ENABLE=1); final return 0
JSON_A='{"version":0,"kind":"Program","body":[
  {"type":"Try","try":[{"type":"Local","name":"x","expr":{"type":"Int","value":1}}],
   "catches":[{"param":"e","typeHint":"Error","body":[{"type":"Local","name":"y","expr":{"type":"Int","value":2}}]}],
   "finally":[{"type":"Local","name":"z","expr":{"type":"Int","value":3}}]
  },
  {"type":"Return","expr":{"type":"Int","value":0}}
]}'
run_json_expect_code "try/catch/finally (accept)" "$JSON_A" 0

# B) break acceptance under dead branch (ignored when not in loop)
JSON_B='{"version":0,"kind":"Program","body":[
  {"type":"If","cond":{"type":"Bool","value":false},"then":[{"type":"Break"}]},
  {"type":"Return","expr":{"type":"Int","value":0}}
]}'
run_json_expect_code "break in dead branch (accept)" "$JSON_B" 0

# C) continue acceptance under dead branch (ignored when not in loop)
JSON_C='{"version":0,"kind":"Program","body":[
  {"type":"If","cond":{"type":"Bool","value":false},"then":[{"type":"Continue"}]},
  {"type":"Return","expr":{"type":"Int","value":0}}
]}'
run_json_expect_code "continue in dead branch (accept)" "$JSON_C" 0

# D) throw acceptance as expression (degrade path unless NYASH_BRIDGE_THROW_ENABLE=1)
JSON_D='{"version":0,"kind":"Program","body":[
  {"type":"Expr","expr":{"type":"Throw","expr":{"type":"Int","value":123}}},
  {"type":"Return","expr":{"type":"Int","value":0}}
]}'
run_json_expect_code "throw (accept)" "$JSON_D" 0

echo "All Stage-3 bridge acceptance smokes PASS" >&2
exit 0

