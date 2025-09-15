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
  local src="$1"; local f="$TMP_DIR/stage2_unary_tmp.ny"
  printf '%s\n' "$src" > "$f"
  NYASH_VM_USE_PY=1 "$BIN" --backend vm "$f" >/dev/null 2>&1 || code=$?
  code=${code:-0}
  echo "__EXIT_CODE__=${code}"
}

OUT=$(run_pyvm_src $'static box Main {\n  main(args){ return -3 + 5 }\n}')
echo "$OUT" | rg -q '^__EXIT_CODE__=2$' && pass "unary minus: -3+5 -> 2" || fail "unary minus" "$OUT"

echo "All Stage-2 unary smokes (PyVM) PASS" >&2

