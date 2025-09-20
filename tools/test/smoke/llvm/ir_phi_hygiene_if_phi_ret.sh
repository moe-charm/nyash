#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/llvm_if_phi_ret.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release --features llvm)" >&2
  exit 1
fi

export NYASH_LLVM_USE_HARNESS=1
export NYASH_LLVM_SANITIZE_EMPTY_PHI=1

irfile="$root/tmp/$(basename "$src" .nyash)_llvm.ll"
mkdir -p "$root/tmp"
NYASH_LLVM_DUMP_IR="$irfile" "$bin" --backend llvm "$src" >/dev/null 2>&1 || true

if [ ! -s "$irfile" ]; then
  echo "[FAIL] IR not dumped for $src" >&2
  exit 2
fi

# No empty phi nodes in IR
empty_cnt=$( (rg -n "\bphi\b" "$irfile" || true) | (rg -v "\[" || true) | wc -l | tr -d ' ' )
if [ "${empty_cnt:-0}" != "0" ]; then
  echo "[FAIL] Empty PHI detected in $irfile" >&2
  rg -n "\bphi\b" "$irfile" | rg -v "\[" || true
  exit 2
fi

echo "[OK] LLVM PHI hygiene (if phi ret) passed"
exit 0

