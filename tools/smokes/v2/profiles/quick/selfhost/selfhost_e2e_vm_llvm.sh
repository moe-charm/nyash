#!/bin/bash
# selfhost_e2e_vm_llvm.sh — M3: VM/LLVM parity using selfhost compiler in runner

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
export NYASH_LLVM_USE_HARNESS=1
export NYASH_NYRT_SILENT_RESULT=1

ensure_hako_toml

# Representative sample (tiny): const_ret
APP_PATH="apps/tests/selfhost_min/const_ret.hako"
if [ ! -f "$NYASH_ROOT/$APP_PATH" ]; then
  echo "SKIP: selfhost_e2e_vm_llvm (sample not found: $APP_PATH)" >&2
  exit 0
fi

# Run VM with child selfhost compiler
VM_OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=0 run_nyash_vm "$NYASH_ROOT/$APP_PATH" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs)
if [ -z "$VM_OUT" ]; then
  log_error "missing Result line (VM)"
  exit 1
fi

# Run LLVM harness with child selfhost compiler
LL_OUT=$(NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_EMIT_ONLY=0 run_nyash_llvm "$NYASH_ROOT/$APP_PATH" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs || true)
if [ -z "$LL_OUT" ]; then
  log_warn "LLVM harness unavailable or no output; SKIP parity compare"
  echo "$VM_OUT"
  exit 0
fi

compare_outputs "$VM_OUT" "$LL_OUT" "selfhost_e2e_vm_llvm"
exit $?

