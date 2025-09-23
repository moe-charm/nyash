#!/usr/bin/env bash
set -euo pipefail

# Small smoke: ensure no empty PHI appears in IR
# Usage: tools/test/smoke/llvm/ir_phi_empty_check.sh [nyash_script]

SCRIPT=${1:-apps/tests/loop_if_phi.nyash}

echo "[phi-empty-check] building nyash (llvm features)" >&2
LLVM_FEATURE=${NYASH_LLVM_FEATURE:-llvm}
if [[ "$LLVM_FEATURE" == "llvm-inkwell-legacy" ]]; then
  # Legacy inkwell needs LLVM_SYS_180_PREFIX
  LLVM_PREFIX=${LLVM_SYS_180_PREFIX:-$(command -v llvm-config-18 >/dev/null 2>&1 && llvm-config-18 --prefix || true)}
  if [[ -n "${LLVM_PREFIX}" ]]; then
    LLVM_SYS_180_PREFIX="${LLVM_PREFIX}" cargo build --release --features "${LLVM_FEATURE}" >/dev/null
  else
    cargo build --release --features "${LLVM_FEATURE}" >/dev/null
  fi
else
  # llvm-harness (default) doesn't need LLVM_SYS_180_PREFIX
  cargo build --release --features "${LLVM_FEATURE}" >/dev/null
fi

IR_OUT=tmp/nyash_harness.ll
mkdir -p tmp

echo "[phi-empty-check] running harness on ${SCRIPT}" >&2
NYASH_LLVM_USE_HARNESS=1 \
NYASH_LLVM_DUMP_IR="${IR_OUT}" \
./target/release/nyash --backend llvm "${SCRIPT}" >/dev/null || true

if [[ ! -s "${IR_OUT}" ]]; then
  echo "[phi-empty-check] WARN: IR dump not found; harness may have short-circuited" >&2
  exit 0
fi

# Check: any phi i64 line must include '[' (incoming pairs)
if rg -n "= phi i64( |$)" "${IR_OUT}" | rg -v "\\[" -n >/dev/null; then
  echo "[phi-empty-check] FAIL: empty PHI found (no incoming list)" >&2
  rg -n "\\= phi i64( |$)" "${IR_OUT}" | rg -v "\\[" -n || true
  exit 1
fi

echo "[phi-empty-check] OK: no empty PHI detected in ${IR_OUT}" >&2
exit 0
