#!/usr/bin/env bash
# Bench: Hakorune VM (nyvm AOT exe) vs Rust VM binary
# For each benchmark source under apps/benchmarks, compile MIR JSON once,
# embed into a tiny Hakorune VM wrapper, build AOT exe via tools/build_llvm.sh,
# then run both nyvm-exe and rust-vm and compare ms/op.
# Usage: tools/bench/bench_nyvm_exe_vs_vm.sh [glob_or_file] [warmup] [repeat]
# Defaults: glob '0?_*.nyash', warmup=2, repeat=20

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

CSV=0
if [[ "${1:-}" == "--csv" ]]; then CSV=1; shift; fi
GLOB="${1:-0?_*.hako}"
WARMUP="${2:-2}"
REPEAT="${3:-20}"
BENCH_DIR="apps/benchmarks"
BIN="${NYASH_BIN:-./target/release/hakorune}"

# Fairness env
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING=0
export HAKO_ALLOW_USING_FILE=1
export RUST_BACKTRACE=0

if [[ "$CSV" -eq 1 ]]; then
  echo "case,nyvm_exe_ms,vm_ms,ratio"
else
  printf "%-28s | %10s | %10s | %6s\n" "case" "nyvm-exe" "vm" "ratio"
  echo "-----------------------------------------------------------------------"
fi

escape_json() {
  # python-based JSON escaper
  python3 - "$1" << 'PY'
import sys, json
path = sys.argv[1]
with open(path, 'r', encoding='utf-8') as f:
    s = f.read()
print(json.dumps(s))
PY
}

build_exe_from_src() {
  local SRC_FILE="$1"; local OUT_EXE="$2"
  timeout 60s env NYASH_LLVM_COMPILER=crate NYASH_LLVM_EMIT=exe NYASH_LLVM_NYRT=target/release \
    tools/build_llvm.sh "$SRC_FILE" -o "$OUT_EXE" >/dev/null 2>&1 || true
}

run_ms_avg() {
  local CMD="$1"; local WARM="$2"; local REP="$3"
  for i in $(seq 1 "$WARM"); do timeout 20s bash -lc "$CMD" >/dev/null 2>&1 || true; done
  local t0 t1; t0=$(date +%s%3N)
  for i in $(seq 1 "$REP"); do timeout 20s bash -lc "$CMD" >/dev/null 2>&1 || true; done
  t1=$(date +%s%3N); echo $(( (t1 - t0) / REP ))
}

shopt -s nullglob
files=("$BENCH_DIR"/$GLOB)
if [ ${#files[@]} -eq 0 ] && [ "${1:-}" = "" ]; then
  GLOB="0?_*.nyash"
  files=("$BENCH_DIR"/$GLOB)
fi
for SRC in "${files[@]}"; do
  CASE="$(basename "$SRC")"
  # 1) Produce MIR JSON
  EXE="/tmp/bench_${CASE%.nyash}_nyvm_exe"
  build_exe_from_src "$SRC" "$EXE"
  if [ ! -x "$EXE" ]; then echo "[warn] build failed: $CASE" >&2; continue; fi
  # 3) Measure
  NYVM_MS=$(run_ms_avg "$EXE" "$WARMUP" "$REPEAT")
  VM_MS=$(run_ms_avg "$BIN --backend vm $SRC" "$WARMUP" "$REPEAT")
  RATIO="-"; if [ "$VM_MS" -gt 0 ]; then RATIO=$(awk "BEGIN{printf \"%.2f\", $NYVM_MS/$VM_MS}"); fi
  if [[ "$CSV" -eq 1 ]]; then
    echo "$CASE,$NYVM_MS,$VM_MS,$RATIO"
  else
    printf "%-28s | %10s | %10s | %6s\n" "$CASE" "$NYVM_MS" "$VM_MS" "$RATIO"
  fi
done
