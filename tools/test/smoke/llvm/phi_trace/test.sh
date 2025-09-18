#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release
# Ensure LLVM harness feature is built (enables object emit via Python harness)
(cd "$ROOT" && cargo build --release --features llvm -j 8 >/dev/null)

export NYASH_LLVM_USE_HARNESS=1
export NYASH_MIR_NO_PHI=${NYASH_MIR_NO_PHI:-1}
export NYASH_VERIFY_ALLOW_NO_PHI=${NYASH_VERIFY_ALLOW_NO_PHI:-1}
export NYASH_LLVM_TRACE_PHI=1
export NYASH_LLVM_PREPASS_IFMERGE=1
export NYASH_LLVM_OBJ_OUT=${NYASH_LLVM_OBJ_OUT:-"$ROOT/tmp/phi_trace_obj.o"}

mkdir -p "$ROOT/tmp"
TRACE_OUT="$ROOT/tmp/phi_trace.jsonl"
rm -f "$TRACE_OUT"
export NYASH_LLVM_TRACE_OUT="$TRACE_OUT"

# Run a couple of representative cases
APP1="$ROOT/apps/tests/loop_if_phi.nyash"
APP2="$ROOT/apps/tests/ternary_nested.nyash"
APP3="$ROOT/apps/tests/llvm_phi_mix.nyash"
APP4="$ROOT/apps/tests/llvm_phi_heavy_mix.nyash"

# Tolerate harness non-zero exits; we validate the trace file instead
timeout -s KILL 30s "$ROOT/target/release/nyash" --backend llvm "$APP1" >/dev/null || true
timeout -s KILL 30s "$ROOT/target/release/nyash" --backend llvm "$APP2" >/dev/null || true
timeout -s KILL 30s "$ROOT/target/release/nyash" --backend llvm "$APP3" >/dev/null || true
timeout -s KILL 30s "$ROOT/target/release/nyash" --backend llvm "$APP4" >/dev/null || true

# Validate trace consistency
assert_exit "python3 \"$ROOT/tools/phi_trace_check.py\" --file \"$TRACE_OUT\" --summary" 0

echo "OK: llvm phi_trace (trace + check)"
