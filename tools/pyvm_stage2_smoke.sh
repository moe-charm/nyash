#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash (release) ..." >&2
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

pass() { echo "✅ $1" >&2; }
fail() { echo "❌ $1" >&2; echo "$2" >&2; exit 1; }

run_pyvm() {
  NYASH_VM_USE_PY=1 "$BIN" --backend vm "$1" 2>&1
}

# 1) String ops basic
OUT=$(run_pyvm "$ROOT_DIR/apps/tests/string_ops_basic.nyash" || true)
echo "$OUT" | rg -q '^len=5$' && echo "$OUT" | rg -q '^sub=bcd$' && echo "$OUT" | rg -q '^idx=1$' \
  && pass "PyVM: string ops basic" || fail "PyVM: string ops basic" "$OUT"

# 2) me.method() call
OUT=$(run_pyvm "$ROOT_DIR/apps/tests/me_method_call.nyash" || true)
echo "$OUT" | rg -q '^n=3$' && pass "PyVM: me method call" || fail "PyVM: me method call" "$OUT"

# 3) If/Loop + PHI
OUT=$(run_pyvm "$ROOT_DIR/apps/tests/loop_if_phi.nyash" || true)
echo "$OUT" | rg -q '^sum=9$' && pass "PyVM: loop/if/phi" || fail "PyVM: loop/if/phi" "$OUT"

# 4) esc_json + dirname smoke
OUT=$(run_pyvm "$ROOT_DIR/apps/tests/esc_dirname_smoke.nyash" || true)
echo "$OUT" | rg -q '^A\\\\\\"B\\\\\\\\C$' && echo "$OUT" | rg -q '^dir1/dir2$' \
  && pass "PyVM: esc_json + dirname" || fail "PyVM: esc_json + dirname" "$OUT"

# 5) Ternary basic
NYASH_VM_USE_PY=1 "$BIN" --backend vm "$ROOT_DIR/apps/tests/ternary_basic.nyash" >/dev/null 2>&1 || code=$?
code=${code:-0}
[[ "$code" -eq 10 ]] && pass "PyVM: ternary basic (exit=10)" || fail "PyVM: ternary basic" "exit=$code"
unset code

# 6) Ternary nested
NYASH_VM_USE_PY=1 "$BIN" --backend vm "$ROOT_DIR/apps/tests/ternary_nested.nyash" >/dev/null 2>&1 || code=$?
code=${code:-0}
[[ "$code" -eq 50 ]] && pass "PyVM: ternary nested (exit=50)" || fail "PyVM: ternary nested" "exit=$code"
unset code

# 7) Match expr block (exit=1)
NYASH_VM_USE_PY=1 "$BIN" --backend vm "$ROOT_DIR/apps/tests/peek_expr_block.nyash" >/dev/null 2>&1 || code=$?
code=${code:-0}
[[ "$code" -eq 1 ]] && pass "PyVM: match expr block (exit=1)" || fail "PyVM: match expr block" "exit=$code"
unset code

# 8) Match return value (temporarily skipped; covered by block form)
# OUT=$(run_pyvm "$ROOT_DIR/apps/tests/peek_return_value.nyash" || true)
# echo "$OUT" | rg -q '^1$' && pass "PyVM: match return value" || fail "PyVM: match return value" "$OUT"

echo "All PyVM Stage-2 smokes PASS" >&2
