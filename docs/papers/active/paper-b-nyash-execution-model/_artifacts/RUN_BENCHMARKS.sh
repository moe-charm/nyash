#!/usr/bin/env bash
set -euo pipefail

# Repro benchmarks for Paper B (Nyash language & execution model)
# Uses the shared benchmarks folder; writes CSVs under _artifacts/results

if ROOT_DIR=$(git -C "$(dirname "$0")" rev-parse --show-toplevel 2>/dev/null); then
  ROOT_DIR="$ROOT_DIR/nyash"
  [[ -d "$ROOT_DIR" ]] || ROOT_DIR=$(git rev-parse --show-toplevel)
else
  ROOT_DIR=$(cd "$(dirname "$0")/../../../../.." && pwd)
fi
ART_DIR=$(cd "$(dirname "$0")" && pwd)
RES_DIR="$ART_DIR/results"
mkdir -p "$RES_DIR"

NYASH_BIN=${NYASH_BIN:-"$ROOT_DIR/target/release/nyash"}
SKIP_INTERP=${SKIP_INTERP:-0}
USE_EXE_ONLY=${USE_EXE_ONLY:-0}
HYPERFINE=$(command -v hyperfine || true)

BENCH_DIR="$ROOT_DIR/benchmarks"
FILES=(
  "$BENCH_DIR/bench_light.nyash"
  "$BENCH_DIR/bench_medium.nyash"
  "$BENCH_DIR/bench_heavy.nyash"
)

echo "[INFO] NYASH_BIN=$NYASH_BIN"
echo "[INFO] USE_EXE_ONLY=$USE_EXE_ONLY (1=EXE only)"
echo "[INFO] hyperfine=${HYPERFINE:-not found}"

if [[ ! -x "$NYASH_BIN" && "$USE_EXE_ONLY" -eq 0 ]]; then
  echo "[INFO] Building nyash (release, with JIT feature)"
  (cd "$ROOT_DIR" && cargo build --release --features cranelift-jit)
fi

have_build_aot=0
if [[ -x "$ROOT_DIR/tools/build_aot.sh" ]]; then
  have_build_aot=1
fi

run_cmd() {
  local cmd="$1" label="$2" csv="$3"
  if [[ -n "$HYPERFINE" ]]; then
    $HYPERFINE -w 2 -r 10 --export-csv "$csv" --show-output --min-runs 10 "$cmd"
  else
    : > "$csv"
    for i in {1..10}; do
      local t0=$(python3 - <<<'import time; print(int(time.time()*1000))')
      bash -lc "$cmd" >/dev/null 2>&1 || true
      local t1=$(python3 - <<<'import time; print(int(time.time()*1000))')
      echo "$label,$((t1-t0))" >> "$csv"
    done
  fi
}

for f in "${FILES[@]}"; do
  [[ -f "$f" ]] || { echo "[WARN] Skip missing $f"; continue; }
  base=$(basename "$f" .nyash)

  if [[ "$USE_EXE_ONLY" -eq 0 ]]; then
    if [[ "$SKIP_INTERP" -eq 0 ]]; then
      run_cmd "$NYASH_BIN $f" "interp-$base" "$RES_DIR/${base}_interp.csv"
    else
      echo "[INFO] SKIP_INTERP=1: skipping interpreter for $f"
    fi
    run_cmd "$NYASH_BIN --backend vm $f" "vm-$base" "$RES_DIR/${base}_vm.csv"
    run_cmd "NYASH_JIT_EXEC=1 $NYASH_BIN --backend vm $f" "jit-$base" "$RES_DIR/${base}_jit.csv"
  fi

  if [[ $have_build_aot -eq 1 ]]; then
    out="/tmp/ny_${base}_aot"
    bash "$ROOT_DIR/tools/build_aot.sh" "$f" -o "$out" >/dev/null 2>&1 || true
    if [[ -x "$out" ]]; then
      run_cmd "$out" "aot-$base" "$RES_DIR/${base}_aot.csv"
      rm -f "$out"
    else
      echo "[WARN] AOT build failed for $f"
    fi
  else
    echo "[INFO] AOT tool not found; skipping AOT for $f"
  fi
done

echo "[DONE] Results in $RES_DIR"
