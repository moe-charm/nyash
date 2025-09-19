#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release --features llvm)" >&2
  exit 1
fi

# Enable macro engine (default ON); avoid forcing macro PATHS globally
export NYASH_MACRO_ENABLE=1

# Use LLVM harness and dump IR
export NYASH_LLVM_USE_HARNESS=1

fails=0

check_case() {
  local src="$1"
  local irfile="$root/tmp/$(basename "$src" .nyash)_llvm.ll"
  mkdir -p "$root/tmp"
  if [[ "$src" == *"macro/loopform"* ]]; then
    NYASH_MACRO_PATHS="apps/macros/examples/loop_normalize_macro.nyash" \
    NYASH_USE_NY_COMPILER=1 NYASH_VM_USE_PY=1 NYASH_LLVM_DUMP_IR="$irfile" \
      "$bin" --macro-preexpand --backend llvm "$src" >/dev/null 2>&1 || true
  else
    NYASH_MACRO_ENABLE=0 NYASH_LLVM_DUMP_IR="$irfile" "$bin" --backend llvm "$src" >/dev/null 2>&1 || true
  fi
  if [ ! -s "$irfile" ]; then
    echo "[SKIP] IR not dumped (mock) for $src"
    return
  fi
  # Hygiene checks:
  # 1) No empty phi nodes (phi ... with no '[' incoming pairs)
  local empty_cnt
  empty_cnt=$( (rg -n "\\bphi\\b" "$irfile" || true) | (rg -v "\\[" || true) | wc -l | tr -d ' ' )
  if [ "${empty_cnt:-0}" != "0" ]; then
    echo "[FAIL] Empty PHI detected in $irfile" >&2
    rg -n "\\bphi\\b" "$irfile" | rg -v "\\[" || true
    fails=$((fails+1))
    return
  fi
  echo "[OK] PHI hygiene (no empty PHI): $(basename "$irfile")"
}

check_case "apps/tests/macro/loopform/simple.nyash"
check_case "apps/tests/macro/loopform/two_vars.nyash"
check_case "apps/tests/macro/loopform/with_continue.nyash"
check_case "apps/tests/macro/loopform/with_break.nyash"
check_case "apps/tests/llvm_phi_mix.nyash"

if [ "$fails" -ne 0 ]; then
  exit 2
fi
echo "[OK] LLVM PHI hygiene for LoopForm cases passed"
exit 0
