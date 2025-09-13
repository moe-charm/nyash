#!/usr/bin/env bash
set -euo pipefail

if [[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]]; then
  set -x
fi

APP="${1:-apps/tests/esc_dirname_smoke.nyash}"
OUT="app_pyvm_cmp"

if [[ ! -f "$APP" ]]; then
  echo "error: app not found: $APP" >&2
  exit 2
fi

# 1) Build nyash with llvm harness enabled (build_llvm.sh does the right thing)
echo "[cmp] building AOT via llvmlite harness ..." >&2
./tools/build_llvm.sh "$APP" -o "$OUT" >/dev/null

# 2) Run AOT executable and capture stdout + exit code
echo "[cmp] running AOT (llvmlite) ..." >&2
set +e
OUT_LL=$("./$OUT" 2>&1)
CODE_LL=$?
set -e

# 3) Run PyVM path (VM mode delegated to Python)
echo "[cmp] running PyVM ..." >&2
set +e
OUT_PY=$(NYASH_VM_USE_PY=1 ./target/release/nyash --backend vm "$APP" 2>&1)
CODE_PY=$?
set -e

echo "=== llvmlite (AOT) stdout ==="
echo "$OUT_LL" | sed -n '1,120p'
echo "=== PyVM stdout ==="
echo "$OUT_PY" | sed -n '1,120p'
echo "=== exit codes ==="
echo "llvmlite: $CODE_LL"
echo "PyVM    : $CODE_PY"

DIFF=0
if [[ "$OUT_LL" != "$OUT_PY" ]]; then
  echo "[cmp] stdout differs" >&2
  DIFF=1
fi
if [[ "$CODE_LL" -ne "$CODE_PY" ]]; then
  echo "[cmp] exit code differs" >&2
  DIFF=1
fi

if [[ "$DIFF" -eq 0 ]]; then
  echo "✅ parity OK (stdout + exit code)"
else
  echo "❌ parity mismatch" >&2
  exit 1
fi

