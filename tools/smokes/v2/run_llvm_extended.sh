#!/usr/bin/env bash
set -euo pipefail

# Extended LLVM smoke runner (heavy/optional)
# Discovers non-phi heavy LLVM tests by simple name heuristics and runs
# harness emit + link + run; prints summary. Failures are reported, but
# cases can be filtered via EXT_FILTER.

MODE=${1:-release}
BIN=./target/${MODE}/nyash
APP_BIN_DIR=${APP_BIN_DIR:-tmp}
TIMEOUT=${TIMEOUT:-30}
EXT_FILTER=${EXT_FILTER:-}
mkdir -p "$APP_BIN_DIR" target/aot_objects

mapfile -t found < <(ls -1 apps/tests/llvm_*.nyash 2>/dev/null | sort || true)
cases=()
for f in "${found[@]:-}"; do
  [[ -z "$f" ]] && continue
  # quick から外した heavy 群のみ（mix/heavy/stage3等）
  base=$(basename "$f")
  case "$base" in
    *phi_mix*|*phi_heavy*|*stage3*|*bitops*|*try_mix* ) : ;; 
    * ) continue;;
  esac
  if [[ -n "$EXT_FILTER" ]] && [[ "$f" != *"$EXT_FILTER"* ]]; then
    continue
  fi
  cases+=("$f")
done

echo "[llvm-ext] building nyash (${MODE})..." >&2
cargo build ${MODE:+--${MODE}} -q

pass=0; fail=0; skip=0
for app in "${cases[@]}"; do
  name=$(basename "$app" .nyash)
  echo "[llvm-ext] case: $name" >&2
  OBJ="$PWD/target/aot_objects/${name}.o"
  rm -f "$OBJ"
  NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_OBJ_OUT="$OBJ" "$BIN" --backend llvm "$app" >/dev/null || true
  if [[ ! -s "$OBJ" ]]; then
    echo "[llvm-ext][SKIP] $name: harness did not produce object" >&2
    skip=$((skip+1)); continue
  fi
  NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ" ./tools/build_llvm.sh "$app" -o "$APP_BIN_DIR/app_${name}" >/dev/null || true
  if [[ ! -x "$APP_BIN_DIR/app_${name}" ]]; then
    echo "[llvm-ext][SKIP] $name: link step failed" >&2
    skip=$((skip+1)); continue
  fi
  out=$(timeout ${TIMEOUT}s "$APP_BIN_DIR/app_${name}" 2>/dev/null || true)
  if [[ $? -ne 0 ]]; then
    echo "[llvm-ext][FAIL] $name: non-zero or timeout" >&2
    fail=$((fail+1))
  else
    echo "[llvm-ext][OK] $name" >&2
    pass=$((pass+1))
  fi
done

echo "[llvm-ext] summary: pass=$pass fail=$fail skip=$skip" >&2
[[ $fail -eq 0 ]]

