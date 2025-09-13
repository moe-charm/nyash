#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
APP=${1:-apps/selfhost/tools/dep_tree_min_string.nyash}
OUTDIR=${OUTDIR:-$ROOT_DIR/tmp}
mkdir -p "$OUTDIR"

ON_EXE=${ON_EXE:-$ROOT_DIR/app_dep_tree_py}
OFF_EXE=${OFF_EXE:-$ROOT_DIR/app_dep_tree_rust}

echo "[compare] target app: $APP"

echo "[compare] build (OFF/Rust LLVM or harness fallback) ..."
# If legacy inkwell backend is not in use, fall back to harness for OFF as well
NYASH_LLVM_SKIP_NYRT_BUILD=1 NYASH_LLVM_USE_HARNESS=1 "$ROOT_DIR/tools/build_llvm.sh" "$APP" -o "$OFF_EXE" >/dev/null

echo "[compare] build (ON/llvmlite harness) ..."
NYASH_LLVM_SKIP_NYRT_BUILD=1 NYASH_LLVM_USE_HARNESS=1 "$ROOT_DIR/tools/build_llvm.sh" "$APP" -o "$ON_EXE" >/dev/null

echo "[compare] run both and capture output ..."
ON_OUT="$OUTDIR/on.out"; OFF_OUT="$OUTDIR/off.out"
set +e
"$ON_EXE" > "$ON_OUT" 2>&1
RC_ON=$?
"$OFF_EXE" > "$OFF_OUT" 2>&1
RC_OFF=$?
set -e

echo "[compare] exit codes: ON=$RC_ON OFF=$RC_OFF"

echo "[compare] extract JSON payload (from first '{' to end) ..."
ON_JSON="$OUTDIR/on.json"; OFF_JSON="$OUTDIR/off.json"
sed -n '/^{/,$p' "$ON_OUT" > "$ON_JSON" || true
sed -n '/^{/,$p' "$OFF_OUT" > "$OFF_JSON" || true

echo "[compare] === diff(json) ==="
diff -u "$OFF_JSON" "$ON_JSON" || true

echo "[compare] files:"
echo "  ON  out: $ON_OUT"
echo "  ON json: $ON_JSON"
echo "  OFF out: $OFF_OUT"
echo "  OFF json: $OFF_JSON"

if [ $RC_ON -eq 0 ] && [ $RC_OFF -eq 0 ]; then
  echo "[compare] ✅ exit codes match (0)"
else
  echo "[compare] ⚠️ exit codes differ or non‑zero (ON=$RC_ON OFF=$RC_OFF)"
fi

# Fallback: if JSON both empty, compare Result: lines
if [ ! -s "$ON_JSON" ] && [ ! -s "$OFF_JSON" ]; then
  echo "[compare] JSON empty on both; compare 'Result:' lines as fallback"
  ON_RES=$(sed -n 's/^.*Result: \(.*\)$/\1/p' "$ON_OUT" | tail -n 1)
  OFF_RES=$(sed -n 's/^.*Result: \(.*\)$/\1/p' "$OFF_OUT" | tail -n 1)
  echo "[compare] ON  Result: ${ON_RES:-<none>}"
  echo "[compare] OFF Result: ${OFF_RES:-<none>}"
  if [ "${ON_RES:-X}" = "${OFF_RES:-Y}" ]; then
    echo "[compare] ✅ fallback results match"
  else
    echo "[compare] ❌ fallback results differ"
  fi
fi
