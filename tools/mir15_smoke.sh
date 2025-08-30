#!/usr/bin/env bash
set -euo pipefail

# MIR15 coverage smoke for VM + JIT (direct)
# Usage: tools/mir15_smoke.sh [debug|release]

MODE=${1:-release}
BIN=./target/${MODE}/nyash

echo "[smoke] building nyash (${MODE}, cranelift-jit)..." >&2
cargo build --features cranelift-jit -q ${MODE:+--${MODE}}

run_vm() {
  local file="$1"
  echo "[VM] $file" >&2
  NYASH_VM_STATS=1 "$BIN" --backend vm "$file" >/dev/null || {
    echo "[VM] FAILED: $file" >&2; exit 1; }
}

run_jit_direct() {
  local file="$1"
  echo "[JIT-DIRECT] $file" >&2
  NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_DUMP=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_THRESHOLD=1 \
    "$BIN" --jit-direct "$file" >/dev/null || {
    echo "[JIT] FAILED: $file" >&2; exit 1; }
}

run_jit_direct_optional() {
  local file="$1"
  echo "[JIT-DIRECT] $file (optional)" >&2
  NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_DUMP=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_THRESHOLD=1 \
    "$BIN" --jit-direct "$file" >/dev/null || {
    echo "[JIT] expected fallback: $file" >&2; return 0; }
}

echo "[smoke] VM core samples" >&2
run_vm examples/include_main.nyash         # Const/Return
run_vm examples/jit_branch_demo.nyash      # Branch/Jump/Phi path (via VM)
run_vm examples/console_demo_simple.nyash  # ExternCall(env.console.log)

echo "[smoke] JIT-direct Core-1" >&2
run_jit_direct examples/jit_arith.nyash                     # BinOp
run_jit_direct examples/jit_compare_i64_boolret.nyash       # Compare/bool ret
run_jit_direct examples/jit_direct_local_store_load.nyash   # Load/Store (local slots)
run_jit_direct examples/jit_branch_demo.nyash               # Branch/Jump/PHI(min)

echo "[smoke] JIT-direct hostcalls (handle-based)" >&2
run_jit_direct_optional examples/jit_array_is_empty.nyash            # any.isEmpty (optional)
run_jit_direct_optional examples/jit_hostcall_array_append.nyash     # array.push (optional)
run_jit_direct_optional examples/jit_map_get_param_hh.nyash          # map.get (optional)

echo "[smoke] OK" >&2
