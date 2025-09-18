#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$ROOT"

export NYASH_LLVM_USE_HARNESS=1
export NYASH_MIR_NO_PHI=${NYASH_MIR_NO_PHI:-1}
export NYASH_VERIFY_ALLOW_NO_PHI=${NYASH_VERIFY_ALLOW_NO_PHI:-1}
export NYASH_LLVM_TRACE_PHI=1
export NYASH_LLVM_PREPASS_IFMERGE=1

mkdir -p tmp
export NYASH_LLVM_TRACE_OUT=${NYASH_LLVM_TRACE_OUT:-"$ROOT/tmp/phi_trace.jsonl"}

echo "[phi-trace] building..." >&2
cargo build --release -j 8 >/dev/null

echo "[phi-trace] running quick smoke (loop_if_phi/ternary_nested/phi_mix/heavy_mix) ..." >&2
bash "$ROOT/tools/test/smoke/llvm/phi_trace/test.sh" >/dev/null

echo "[phi-trace] checking trace ..." >&2
python3 "$ROOT/tools/phi_trace_check.py" --file "$NYASH_LLVM_TRACE_OUT" --summary
echo "[phi-trace] OK" >&2

