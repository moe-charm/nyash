#!/usr/bin/env bash
# Hakorune VM (nyvm) vs Rust VM microbench harness (apps/benchmarks/*.nyash)
# Usage:
#   tools/bench/bench_nyvm_vs_vm.sh [glob_or_file] [warmup] [repeat]
# Defaults: glob '*.nyash', warmup=2, repeat=10

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

GLOB="${1:-*.hako}"
WARMUP="${2:-2}"
REPEAT="${3:-10}"

BENCH_DIR="apps/benchmarks"
BIN="${NYASH_BIN:-./target/release/hakorune}"

# Fairness env
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING=0
export HAKO_ALLOW_USING_FILE=1
export RUST_BACKTRACE=0

printf "%-28s | %8s | %8s | %6s\n" "case" "nyvm(ms)" "vm(ms)" "ratio"
echo "---------------------------------------------------------------"

shopt -s nullglob
files=("$BENCH_DIR"/$GLOB)
if [ ${#files[@]} -eq 0 ] && [ "${1:-}" = "" ]; then
  GLOB="*.nyash"
  files=("$BENCH_DIR"/$GLOB)
fi
for f in "${files[@]}"; do
  case_name="$(basename "$f")"
  # Warmup
  for i in $(seq 1 "$WARMUP"); do
    HAKO_NYVM_ENGINE=hakorune "$BIN" --backend nyvm "$f" >/dev/null 2>&1 || true
    "$BIN" --backend vm "$f" >/dev/null 2>&1 || true
  done
  # Measure nyvm
  t0=$(date +%s%3N)
  for i in $(seq 1 "$REPEAT"); do
    HAKO_NYVM_ENGINE=hakorune "$BIN" --backend nyvm "$f" >/dev/null 2>&1 || true
  done
  t1=$(date +%s%3N)
  nyvm_ms=$(( (t1 - t0) / REPEAT ))
  # Measure vm
  t2=$(date +%s%3N)
  for i in $(seq 1 "$REPEAT"); do
    "$BIN" --backend vm "$f" >/dev/null 2>&1 || true
  done
  t3=$(date +%s%3N)
  vm_ms=$(( (t3 - t2) / REPEAT ))
  ratio="-"
  if [ "$vm_ms" -gt 0 ]; then ratio=$(awk "BEGIN{printf \"%.2f\", $nyvm_ms/$vm_ms}"); fi
  printf "%-28s | %8s | %8s | %6s\n" "$case_name" "$nyvm_ms" "$vm_ms" "$ratio"
done
