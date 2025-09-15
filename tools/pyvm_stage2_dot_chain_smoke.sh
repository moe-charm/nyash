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

run_pyvm_src() {
  local src="$1"; local file="$TMP_DIR/stage2_dot_tmp.ny"
  printf '%s\n' "$src" > "$file"
  local out code
  out=$(NYASH_VM_USE_PY=1 "$BIN" --backend vm "$file" 2>&1) || code=$?
  code=${code:-0}
  printf '%s\n__EXIT_CODE__=%s\n' "$out" "$code"
}

SRC=$'static box Main {\n  main(args){\n    return ("abcde").substring(1,4).length()\n  }\n}'
OUT=$(run_pyvm_src "$SRC")
echo "$OUT" | rg -q '^__EXIT_CODE__=3$' \
  && pass "dot-chain: substring.length exit=3" || fail "dot-chain" "$OUT"

echo "All Stage-2 dot-chain smokes (PyVM) PASS" >&2

