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

# ArrayBox minimal ops
OUT=$(run_pyvm "$ROOT_DIR/apps/tests/array_min_ops.nyash" || true)
echo "$OUT" | rg -q '^alen=2$' && echo "$OUT" | rg -q '^g0=10$' && echo "$OUT" | rg -q '^g1=20$' && echo "$OUT" | rg -q '^g1b=30$' \
  && pass "PyVM: ArrayBox minimal ops" || fail "PyVM: ArrayBox minimal ops" "$OUT"

# MapBox minimal ops
OUT=$(run_pyvm "$ROOT_DIR/apps/tests/map_min_ops.nyash" || true)
echo "$OUT" | rg -q '^msz=2$' && echo "$OUT" | rg -q '^ha=1$' && echo "$OUT" | rg -q '^hc=0$' && echo "$OUT" | rg -q '^ga=1$' \
  && pass "PyVM: MapBox minimal ops" || fail "PyVM: MapBox minimal ops" "$OUT"

echo "All PyVM collections smokes PASS" >&2

