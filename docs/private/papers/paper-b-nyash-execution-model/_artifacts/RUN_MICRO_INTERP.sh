#!/usr/bin/env bash
set -euo pipefail

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
BIN="$ROOT/nyash/target/release/nyash"

need() { command -v "$1" >/dev/null 2>&1 || { echo "error: missing: $1" >&2; exit 1; }; }
need "$BIN" || (cd "$ROOT/nyash" && cargo build --release >/dev/null)

declare -A LOOPS
LOOPS[bench_box_create_destroy.nyash]=1000000
LOOPS[bench_method_call_only.nyash]=2000000

bench() {
  local file="$1" loops="$2"
  local path="$ROOT/nyash/benchmarks/$file"
  [[ -f "$path" ]] || { echo "[skip] missing $path"; return; }
  local t0=$(python3 - <<<'import time; print(time.time())')
  "$BIN" "$path" >/dev/null 2>&1 || true
  local t1=$(python3 - <<<'import time; print(time.time())')
  local ms=$(python3 - <<EOF
import sys
print(int( (float($t1) - float($t0)) * 1000 ))
EOF
)
  # ns per op (approx)
  local ns=$(python3 - <<EOF
loops=$loops
ms=$ms
print(int( (ms*1_000_000.0) / loops ))
EOF
)
  echo "$file: ${ms}ms total, ~${ns} ns/op"
}

bench bench_box_create_destroy.nyash ${LOOPS[bench_box_create_destroy.nyash]}
bench bench_method_call_only.nyash ${LOOPS[bench_method_call_only.nyash]}

