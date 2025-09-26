#!/usr/bin/env bash
set -euo pipefail

# Reproducible benchmarks for MIR13 paper (Interpreter/VM/JIT/AOT if available)
# Outputs CSVs under _artifacts/results/

if ROOT_DIR=$(git -C "$(dirname "$0")" rev-parse --show-toplevel 2>/dev/null); then
  ROOT_DIR="$ROOT_DIR/nyash"
  [[ -d "$ROOT_DIR" ]] || ROOT_DIR=$(git rev-parse --show-toplevel)
else
  # Fallback: ascend to repo root from _artifacts
  ROOT_DIR=$(cd "$(dirname "$0")/../../../../.." && pwd)
fi
ART_DIR=$(cd "$(dirname "$0")" && pwd)
RES_DIR="$ART_DIR/results"
mkdir -p "$RES_DIR"

NYASH_BIN=${NYASH_BIN:-"$ROOT_DIR/target/release/nyash"}
SKIP_INTERP=${SKIP_INTERP:-0}     # 1: skip interpreter（遅い環境向け）
USE_LLVM_AOT=${USE_LLVM_AOT:-0}   # 1: LLVM backendでAOTも計測
SKIP_AOT=${SKIP_AOT:-0}           # 1: tools/build_aot.sh 経由のAOT計測をスキップ
RUNS=${RUNS:-10}                  # 計測回数
USE_EXE_ONLY=${USE_EXE_ONLY:-0}   # 1: measure AOT exe only
HYPERFINE=$(command -v hyperfine || true)
TIMEOUT_SECS=${TIMEOUT_SECS:-0}   # >0 なら各コマンドをtimeoutでラップ
TIMEOUT_BIN=$(command -v timeout || true)

BENCH_DIR="$ROOT_DIR/benchmarks"
FILES=(
  "$BENCH_DIR/bench_light.nyash"
  "$BENCH_DIR/bench_medium.nyash"
  "$BENCH_DIR/bench_heavy.nyash"
  "$BENCH_DIR/bench_aot_len_light.nyash"
  "$BENCH_DIR/bench_aot_len_medium.nyash"
  "$BENCH_DIR/bench_aot_len_heavy.nyash"
  "$ROOT_DIR/examples/aot_min_string_len.nyash"
  # Pure arithmetic (no plugins)
  "$BENCH_DIR/bench_arith_pure_light.nyash"
  "$BENCH_DIR/bench_arith_pure_medium.nyash"
  "$BENCH_DIR/bench_arith_pure_heavy.nyash"
)

FILTER_REGEX=${FILTER:-}

echo "[INFO] NYASH_BIN=$NYASH_BIN"
echo "[INFO] USE_EXE_ONLY=$USE_EXE_ONLY (1=EXE only)"
echo "[INFO] hyperfine=${HYPERFINE:-not found}"
echo "[INFO] USE_LLVM_AOT=$USE_LLVM_AOT (1=measure LLVM AOT)"
echo "[INFO] SKIP_AOT=$SKIP_AOT (1=skip AOT via tools/build_aot.sh)"
echo "[INFO] RUNS=$RUNS"

if [[ ! -x "$NYASH_BIN" && "$USE_EXE_ONLY" -eq 0 ]]; then
  echo "[INFO] Building nyash (release, with JIT feature)"
  (cd "$ROOT_DIR" && cargo build --release --features cranelift-jit)
fi

have_build_aot=0
if [[ -x "$ROOT_DIR/tools/build_aot.sh" ]]; then
  have_build_aot=1
fi

have_build_llvm=0
if [[ -x "$ROOT_DIR/tools/build_llvm.sh" ]]; then
  have_build_llvm=1
fi

run_cmd() {
  local cmd="$1" label="$2" csv="$3"
  local cmd_wrap="$cmd"
  if [[ -n "$TIMEOUT_BIN" && "$TIMEOUT_SECS" -gt 0 ]]; then
    cmd_wrap="$TIMEOUT_BIN ${TIMEOUT_SECS}s $cmd"
  fi
  if [[ -n "$HYPERFINE" ]]; then
    # runs configurable, warmup 1, export CSV
    $HYPERFINE -w 1 -r "$RUNS" --export-csv "$csv" --show-output "$cmd_wrap"
  else
    # Simple fallback: run 10 times and record naive timing (ms)
    : > "$csv"
    for i in $(seq 1 "$RUNS"); do
      local t0=$(python3 - <<<'import time; print(int(time.time()*1000))')
      bash -lc "$cmd_wrap" >/dev/null 2>&1 || true
      local t1=$(python3 - <<<'import time; print(int(time.time()*1000))')
      echo "$label,$((t1-t0))" >> "$csv"
    done
  fi
}

# Measure modes
for f in "${FILES[@]}"; do
  if [[ -n "$FILTER_REGEX" ]]; then
    if [[ ! "$f" =~ $FILTER_REGEX ]]; then
      echo "[INFO] FILTER: skip $f"
      continue
    fi
  fi
  [[ -f "$f" ]] || { echo "[WARN] Skip missing $f"; continue; }
  base=$(basename "$f" .nyash)

  if [[ "$USE_EXE_ONLY" -eq 0 ]]; then
    # Interpreter
    if [[ "$SKIP_INTERP" -eq 0 ]]; then
      run_cmd "$NYASH_BIN $f" "interp-$base" "$RES_DIR/${base}_interp.csv"
    else
      echo "[INFO] SKIP_INTERP=1: skipping interpreter for $f"
    fi
    # VM
    run_cmd "$NYASH_BIN --backend vm $f" "vm-$base" "$RES_DIR/${base}_vm.csv"
    # JIT (VM + JIT execute)
    run_cmd "NYASH_JIT_EXEC=1 $NYASH_BIN --backend vm $f" "jit-$base" "$RES_DIR/${base}_jit.csv"
  fi

  # AOT (if tool available)
  if [[ $have_build_aot -eq 1 && "$SKIP_AOT" -eq 0 ]]; then
    out="/tmp/ny_${base}_aot"
    bash "$ROOT_DIR/tools/build_aot.sh" "$f" -o "$out" >/dev/null 2>&1 || true
    if [[ -x "$out" ]]; then
      run_cmd "$out" "aot-$base" "$RES_DIR/${base}_aot.csv"
      rm -f "$out"
    else
      echo "[WARN] AOT build failed for $f"
    fi
  else
    if [[ "$SKIP_AOT" -eq 1 ]]; then
      echo "[INFO] SKIP_AOT=1: skipping AOT for $f"
    else
      echo "[INFO] AOT tool not found; skipping AOT for $f"
    fi
  fi
done

# LLVM AOT-only targets (optional)
if [[ "$USE_LLVM_AOT" -eq 1 ]]; then
  if [[ $have_build_llvm -eq 0 ]]; then
    echo "[WARN] tools/build_llvm.sh not found; skipping LLVM AOT"
  elif ! command -v llvm-config-18 >/dev/null 2>&1; then
    echo "[WARN] llvm-config-18 not found; skipping LLVM AOT"
  else
    LLVM_FILES=(
      "$ROOT_DIR/apps/tests/ny-llvm-smoke/main.nyash"
    )
    for f in "${LLVM_FILES[@]}"; do
      [[ -f "$f" ]] || { echo "[WARN] Skip missing LLVM target $f"; continue; }
      base=$(basename "$f" .nyash)
      out="/tmp/ny_${base}_llvm"
      # Build via LLVM backend
      LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) \
      LLVM_SYS_181_PREFIX=$(llvm-config-18 --prefix) \
      "$ROOT_DIR/tools/build_llvm.sh" "$f" -o "$out" >/dev/null 2>&1 || true
      if [[ -x "$out" ]]; then
        run_cmd "$out" "llvm-aot-$base" "$RES_DIR/${base}_llvm_aot.csv"
        rm -f "$out"
      else
        echo "[WARN] LLVM AOT build failed for $f"
      fi
    done
  fi
fi

# JIT-AOT (Cranelift) via --jit-direct (optional)
USE_JIT_AOT=${USE_JIT_AOT:-0}
echo "[INFO] USE_JIT_AOT=$USE_JIT_AOT (1=measure JIT AOT via jit-direct)"
if [[ "$USE_JIT_AOT" -eq 1 ]]; then
  echo "[JIT-AOT] Building nyash + nyrt ..."
  (cd "$ROOT_DIR" && cargo build --release --features cranelift-jit >/dev/null)
  (cd "$ROOT_DIR/crates/nyrt" && cargo build --release >/dev/null)

  JIT_AOT_FILES=(
    "$ROOT_DIR/apps/examples/array_p0.nyash"
  )
  for f in "${JIT_AOT_FILES[@]}"; do
    [[ -f "$f" ]] || { echo "[WARN] Skip missing JIT-AOT target $f"; continue; }
    base=$(basename "$f" .nyash)
    objdir="$ROOT_DIR/target/aot_objects"
    rm -rf "$objdir" && mkdir -p "$objdir"
    # Emit object via JIT-direct (relaxed)
    NYASH_JIT_EVENTS=1 NYASH_AOT_OBJECT_OUT="$objdir/main.o" "$NYASH_BIN" --jit-direct "$f" >/dev/null || true
    if [[ -f "$objdir/main.o" ]]; then
      out="/tmp/ny_${base}_jit_aot"
      cc "$objdir/main.o" \
        -L "$ROOT_DIR/target/release" \
        -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
        -lpthread -ldl -lm -o "$out"
      if [[ -x "$out" ]]; then
        run_cmd "$out" "jit-aot-$base" "$RES_DIR/${base}_jit_aot.csv"
        rm -f "$out"
      else
        echo "[WARN] link failed for JIT-AOT target $f"
      fi
    else
      echo "[WARN] JIT AOT object not generated for $f"
    fi
  done
fi

echo "[DONE] Results in $RES_DIR"
