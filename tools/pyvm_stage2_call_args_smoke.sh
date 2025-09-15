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

run_exit_code() {
  local src="$1"; local f="$TMP_DIR/stage2_call_args_tmp.ny"
  printf '%s\n' "$src" > "$f"
  NYASH_VM_USE_PY=1 "$BIN" --backend vm "$f" >/dev/null 2>&1 || code=$?
  echo ${code:-0}
}

# Nested args: substring with expression argument
SRC1=$'static box Main {\n  main(args){\n    return ("abcdef").substring(1, 1+2).length()\n  }\n}'
CODE=$(run_exit_code "$SRC1")
[[ "$CODE" -eq 2 ]] && pass "call args: substring(1,1+2).length -> 2" || fail "call args: nested expr arg" "__EXIT_CODE__=$CODE"

# Nested chain with nested calls in args (single line)
SRC2=$'static box Main {\n  main(args){\n    return ("abcdef").substring(1, 1+3).substring(0,2).length()\n  }\n}'
CODE=$(run_exit_code "$SRC2")
[[ "$CODE" -eq 2 ]] && pass "call args: nested calls and expr args -> 2" || fail "call args: nested chain" "__EXIT_CODE__=$CODE"

echo "All Stage-2 call/args smokes (PyVM) PASS" >&2
