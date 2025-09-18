#!/usr/bin/env bash
set -euo pipefail
# Bridge → MIR(JSON) → ny-llvmc (llvmlite harness) with PHI trace
# Usage: tools/phi_trace_bridge_try.sh <json_v0_file>

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <tests/json_v0_stage3/*.json>" >&2
  exit 2
fi

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIN="$ROOT/target/release/nyash"
NYLL="$ROOT/target/release/ny-llvmc"

echo "[bridge-phi] building nyash + ny-llvmc ..." >&2
cargo build --release -q
cargo build --release -p nyash-llvm-compiler -q

JSON_V0="$1"
TMP_DIR="$ROOT/tmp"
mkdir -p "$TMP_DIR"
MIR_JSON="$TMP_DIR/nyash_pyvm_mir.json"
EXE_OUT="$TMP_DIR/bridge_phi_try_exe"
TRACE_OUT="$TMP_DIR/bridge_phi_try.jsonl"
rm -f "$MIR_JSON" "$EXE_OUT" "$TRACE_OUT"

echo "[bridge-phi] lowering JSON v0 → MIR(JSON) via Bridge (pipe)" >&2
# Pipe JSON v0 to nyash; enable PyVM path to force MIR(JSON) emit for harness
NYASH_MIR_NO_PHI=1 NYASH_VERIFY_ALLOW_NO_PHI=1 NYASH_TRY_RESULT_MODE=1 NYASH_PIPE_USE_PYVM=1 \
  "$BIN" --ny-parser-pipe --backend vm < "$JSON_V0" >/dev/null || true
if [[ ! -s "$MIR_JSON" ]]; then
  echo "error: MIR JSON not found: $MIR_JSON" >&2
  exit 1
fi

echo "[bridge-phi] compiling via ny-llvmc (llvmlite harness) with PHI trace" >&2
NYASH_LLVM_TRACE_PHI=1 NYASH_LLVM_PREPASS_IFMERGE=1 NYASH_LLVM_TRACE_OUT="$TRACE_OUT" \
  "$NYLL" --in "$MIR_JSON" --emit exe --nyrt "$ROOT/target/release" --out "$EXE_OUT" --harness "$ROOT/tools/llvmlite_harness.py" >/dev/null

if [[ ! -s "$TRACE_OUT" ]]; then
  echo "error: PHI trace not found: $TRACE_OUT" >&2
  exit 1
fi

echo "[bridge-phi] checking PHI trace consistency" >&2
python3 "$ROOT/tools/phi_trace_check.py" --file "$TRACE_OUT" --summary
echo "[bridge-phi] OK" >&2
