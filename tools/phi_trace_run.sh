#!/usr/bin/env bash
set -euo pipefail

# One-shot: run a .nyash with LLVM harness, emit PHI JSONL trace, and check consistency.
# Usage:
#   tools/phi_trace_run.sh <app.nyash> [app2.nyash ...] [--strict-zero]
# Env (optional):
#   NYASH_LLVM_TRACE_OUT=tmp/phi.jsonl (default under repo tmp/)

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <app.nyash> [app2.nyash ...] [--strict-zero]" >&2
  exit 2
fi

STRICT=0
APPS=()
for a in "$@"; do
  if [[ "$a" == "--strict-zero" ]]; then
    STRICT=1
  else
    APPS+=("$a")
  fi
done
if [[ ${#APPS[@]} -eq 0 ]]; then
  echo "error: no app specified" >&2
  exit 2
fi

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

export NYASH_LLVM_USE_HARNESS=1
export NYASH_MIR_NO_PHI=${NYASH_MIR_NO_PHI:-1}
export NYASH_VERIFY_ALLOW_NO_PHI=${NYASH_VERIFY_ALLOW_NO_PHI:-1}
export NYASH_LLVM_TRACE_PHI=1
export NYASH_LLVM_PREPASS_IFMERGE=${NYASH_LLVM_PREPASS_IFMERGE:-1}

mkdir -p tmp
TRACE_OUT_DEFAULT="$ROOT/tmp/phi_trace_oneshot.jsonl"
export NYASH_LLVM_TRACE_OUT=${NYASH_LLVM_TRACE_OUT:-"$TRACE_OUT_DEFAULT"}
rm -f "$NYASH_LLVM_TRACE_OUT"
export NYASH_LLVM_OBJ_OUT=${NYASH_LLVM_OBJ_OUT:-"$ROOT/tmp/phi_trace_oneshot.o"}

echo "[phi-trace] building nyash (release, llvm harness) ..." >&2
cargo build --release --features llvm -j 8 >/dev/null
echo "[phi-trace] building ny-llvmc (release) ..." >&2
cargo build --release -p nyash-llvm-compiler -j 8 >/dev/null

for APP in "${APPS[@]}"; do
  echo "[phi-trace] running: $APP" >&2
  set +e
  "$ROOT/target/release/nyash" --backend llvm "$APP"
  RC=$?
  set -e
  echo "[phi-trace] nyash exit code: $RC (ignored for trace check)" >&2
done

if [[ ! -s "$NYASH_LLVM_TRACE_OUT" ]]; then
  echo "[phi-trace] error: trace not found: $NYASH_LLVM_TRACE_OUT" >&2
  exit 1
fi

echo "[phi-trace] checking trace: $NYASH_LLVM_TRACE_OUT" >&2
if [[ $STRICT -eq 1 ]]; then
  python3 "$ROOT/tools/phi_trace_check.py" --file "$NYASH_LLVM_TRACE_OUT" --summary --strict-zero
else
  python3 "$ROOT/tools/phi_trace_check.py" --file "$NYASH_LLVM_TRACE_OUT" --summary
fi
echo "[phi-trace] OK" >&2
