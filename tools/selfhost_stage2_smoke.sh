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

run_case_expect() {
  local name="$1"; shift
  local src="$1"; shift
  local regex="$1"; shift
  local file="$TMP/selfhost_${name}.nyash"
  printf "%s\n" "$src" > "$file"
  set +e
  OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=0 NYASH_VM_USE_PY=${NYASH_VM_USE_PY:-1} \
        "$BIN" --backend vm "$file" 2>&1)
  RC=$?
  set -e
  if echo "$OUT" | rg -q "$regex"; then pass "$name"; else fail "$name" "$OUT"; fi
}

# A) arithmetic
run_case_expect "arith" 'return 1+2*3' '^Result:\s*7\b'

# B) unary minus precedence (-3 + 5 -> 2)
run_case_expect "unary" 'return -3 + 5' '^Result:\s*2\b'

# C) logical AND
run_case_expect "and" 'return (1 < 2) && (2 < 3)' '^Result:\s*true\b'

# D) logical OR
run_case_expect "or" 'return (1 > 2) || (2 < 3)' '^Result:\s*true\b'

# E) compare eq
run_case_expect "eq" 'return (1 + 1) == 2' '^Result:\s*true\b'

# F) nested if with locals -> 200
cat > "$TMP/selfhost_nested_if.nyash" <<'NY'
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
set +e
OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=0 NYASH_VM_USE_PY=${NYASH_VM_USE_PY:-1} \
      "$BIN" --backend vm "$TMP/selfhost_nested_if.nyash" 2>&1)
set -e
echo "$OUT" | rg -q '^Result:\s*200\b' && pass "nested if" || fail "nested if" "$OUT"

# G) if/else separated by newline
cat > "$TMP/selfhost_if_else_line.nyash" <<'NY'
local x = 0
if 1 < 2 {
  local x = 10
}
else {
  local x = 20
}
return x
NY
set +e
OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=0 NYASH_VM_USE_PY=${NYASH_VM_USE_PY:-1} \
      "$BIN" --backend vm "$TMP/selfhost_if_else_line.nyash" 2>&1)
set -e
echo "$OUT" | rg -q '^Result:\s*10\b' && pass "if/else separation" || fail "if/else separation" "$OUT"

# H) ArrayBox minimal: push + size -> 2
cat > "$TMP/selfhost_array_ops.nyash" <<'NY'
local a = new ArrayBox()
a.push(1)
a.push(2)
return a.size()
NY
set +e
OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=0 NYASH_VM_USE_PY=${NYASH_VM_USE_PY:-1} NYASH_DISABLE_PLUGINS=1 \
      "$BIN" --backend vm "$TMP/selfhost_array_ops.nyash" 2>&1)
set -e
echo "$OUT" | rg -q '^Result:\s*2\b' && pass "ArrayBox push/size" || fail "ArrayBox push/size" "$OUT"

# I) String.length via method on var -> 3
cat > "$TMP/selfhost_string_len.nyash" <<'NY'
local s = "abc"
return s.length()
NY
set +e
OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=0 NYASH_VM_USE_PY=${NYASH_VM_USE_PY:-1} \
      "$BIN" --backend vm "$TMP/selfhost_string_len.nyash" 2>&1)
set -e
echo "$OUT" | rg -q '^Result:\s*3\b' && pass "String.length()" || fail "String.length()" "$OUT"

echo "All selfhost Stage-2 smokes PASS" >&2
exit 0

