#!/usr/bin/env bash
set -euo pipefail

# Time-boxed interpreter microbenchmarks.
# Runs each benchmark program repeatedly for TIME_SECS seconds and reports ~ns/op.

ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
BIN="$ROOT/nyash/target/release/nyash"
TIME_SECS=${TIME_SECS:-3}

declare -A LOOPS
# Use small-loop variants so each run completes quickly in interpreter
LOOPS[bench_box_create_destroy_small.nyash]=10000
LOOPS[bench_method_call_only_small.nyash]=5000

ensure_bin() {
  if [[ ! -x "$BIN" ]]; then
    echo "[build] nyash (release)"
    (cd "$ROOT/nyash" && cargo build --release >/dev/null)
  fi
}

run_timeboxed() {
  local file="$1" loops_per_run="$2"
  local path="$ROOT/nyash/benchmarks/$file"
  [[ -f "$path" ]] || { echo "[skip] missing $path"; return; }
  local runs=0
  local t0=$(python3 - <<<'import time; print(time.time())')
  local deadline=$(python3 - <<EOF
import time
print(time.time() + $TIME_SECS)
EOF
)
  while :; do
    "$BIN" "$path" >/dev/null 2>&1 || true
    runs=$((runs+1))
    local now=$(python3 - <<<'import time; print(time.time())')
    awk "BEGIN{exit !(($now) >= ($deadline))}" && break || true
  done
  local t1=$(python3 - <<<'import time; print(time.time())')
  local elapsed_ms=$(python3 - <<EOF
print(int( (float($t1) - float($t0)) * 1000 ))
EOF
)
  local total_ops=$((runs * loops_per_run))
  if [[ "$total_ops" -le 0 ]]; then
    echo "$file: no runs completed"
    return
  fi
  local ns_per_op=$(python3 - <<EOF
ops=$total_ops
ms=$elapsed_ms
print(int((ms*1_000_000.0)/ops))
EOF
)
  echo "$file: ${elapsed_ms}ms, runs=$runs, ops=$total_ops, ~${ns_per_op} ns/op"
}

ensure_bin
echo "[time-box] TIME_SECS=$TIME_SECS"
run_timeboxed bench_box_create_destroy_small.nyash ${LOOPS[bench_box_create_destroy_small.nyash]}
run_timeboxed bench_method_call_only_small.nyash ${LOOPS[bench_method_call_only_small.nyash]}
