#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release --features llvm)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/if_match_normalize_macro.nyash"
export NYASH_LLVM_USE_HARNESS=1

fails=0

check_case() {
  local src="$1"
  local irfile="$root/tmp/$(basename "$src" .nyash)_llvm.ll"
  mkdir -p "$root/tmp"
  NYASH_LLVM_DUMP_IR="$irfile" "$bin" --backend llvm "$src" >/dev/null 2>&1 || {
    echo "[FAIL] LLVM run failed for $src" >&2
    fails=$((fails+1))
    return
  }
  if [ ! -s "$irfile" ]; then
    echo "[FAIL] IR not dumped for $src" >&2
    fails=$((fails+1))
    return
  }
  local empty_cnt
  empty_cnt=$(rg -n "\\bphi\\b" "$irfile" | rg -v "\\[" | wc -l | tr -d ' ')
  if [ "${empty_cnt:-0}" != "0" ]; then
    echo "[FAIL] Empty PHI detected in $irfile" >&2
    rg -n "\\bphi\\b" "$irfile" | rg -v "\\[" || true
    fails=$((fails+1))
    return
  fi
  echo "[OK] PHI hygiene (no empty PHI): $(basename "$irfile")"
}

check_case "apps/tests/macro_golden_if_assign.nyash"
check_case "apps/tests/macro_golden_if_print.nyash"

if [ "$fails" -ne 0 ]; then
  exit 2
fi
echo "[OK] LLVM PHI hygiene for If-cases passed"
exit 0

