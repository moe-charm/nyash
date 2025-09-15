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
  local src="$1"; local f="$TMP_DIR/stage2_nested_tmp.ny"
  printf '%s\n' "$src" > "$f"
  NYASH_VM_USE_PY=1 "$BIN" --backend vm "$f" >/dev/null 2>&1 || code=$?
  code=${code:-0}
  echo "__EXIT_CODE__=${code}"
}

# sum 1..5 -> 15; with nested if filtering even numbers to a separate counter
SRC=$'static box Main {\n  main(args){\n    local i = 1\n    local sum = 0\n    loop(i <= 5) {\n      if (i % 2 == 0) {\n        sum = sum + 0\n      } else {\n        sum = sum + i\n      }\n      i = i + 1\n    }\n    return sum  // 1+3+5 = 9\n  }\n}'
OUT=$(run_pyvm_src "$SRC")
echo "$OUT" | rg -q '^__EXIT_CODE__=9$' && pass "nested control: loop+if -> exit=9" || fail "nested control" "$OUT"

echo "All Stage-2 nested control smokes (PyVM) PASS" >&2
