#!/usr/bin/env bash
set -euo pipefail

# build_and_run_wasm.sh — One-shot: .nyash/.hako → MIR JSON → WASM → Run (exit code)
# Requirements: cargo-built nyash CLI, python3+llvmlite, node (for wasm_runner.js)

if [ $# -lt 1 ]; then
  echo "Usage: $0 <program.nyash|program.hako> [--keep]" >&2
  exit 2
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${NYASH_BIN:-$ROOT/target/release/nyash}"
PROG="$1"
KEEP="0"
shift || true
if [ "${1:-}" = "--keep" ]; then KEEP="1"; shift; fi

if [ ! -x "$BIN" ]; then
  echo "[info] Building CLI ..." >&2
  (cd "$ROOT" && cargo build --release)
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 not found (needed for llvmlite harness)" >&2
  exit 3
fi
python3 - <<'PY' || { echo "error: llvmlite not available (pip install llvmlite)" >&2; exit 3; }
import sys
try:
  import llvmlite # noqa
except Exception as e:
  sys.exit(2)
print('ok')
PY

if ! command -v node >/dev/null 2>&1; then
  echo "error: node not found (needed to run wasm)" >&2
  exit 3
fi

TMP="/tmp/nyash_build_wasm_$$"
mkdir -p "$TMP"
JSON="$TMP/out.json"
WASM="$TMP/out.wasm"

echo "[step] Emit MIR JSON → $JSON" >&2
"$BIN" --emit-mir-json "$JSON" "$PROG"

echo "[step] Build WASM object → $WASM" >&2
PYTHONPATH="$ROOT" python3 "$ROOT/src/llvm_py/llvm_builder.py" \
  --target wasm32 "$JSON" -o "$WASM"

echo "[step] Run WASM" >&2
node "$ROOT/src/llvm_py/tools/wasm_runner.js" "$WASM"
EC=$?
echo "[result] exit code: $EC" >&2

if [ "$KEEP" != "1" ]; then rm -rf "$TMP"; fi
exit "$EC"

