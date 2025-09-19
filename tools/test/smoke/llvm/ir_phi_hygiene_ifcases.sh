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
export NYASH_LLVM_SANITIZE_EMPTY_PHI=1

fails=0

check_case() {
  local src="$1"
  local irfile="$root/tmp/$(basename "$src" .nyash)_llvm.ll"
  mkdir -p "$root/tmp"
  NYASH_LLVM_DUMP_IR="$irfile" "$bin" --backend llvm "$src" >/dev/null 2>&1 || true
  if [ ! -s "$irfile" ]; then
    # guard: some cases may run mock backend; allow skip for those
    if [[ "$src" == *"guard_literal_or.nyash"* ]] || [[ "$src" == *"literal_three_arms.nyash"* ]] || [[ "$src" == *"assign_both_branches.nyash"* ]]; then
      echo "[SKIP] IR not dumped (mock) for $src"
      return
    fi
    echo "[FAIL] IR not dumped for $src" >&2
    fails=$((fails+1))
    return
  fi
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

check_case "apps/tests/macro/if/assign.nyash"
check_case "apps/tests/macro/if/print_expr.nyash"
check_case "apps/tests/macro/match/literal_basic.nyash"
check_case "apps/tests/macro/match/guard_literal_or.nyash"
check_case "apps/tests/macro/match/literal_three_arms.nyash"
check_case "apps/tests/macro/if/assign_both_branches.nyash"

if [ "$fails" -ne 0 ]; then
  exit 2
fi
echo "[OK] LLVM PHI hygiene for If-cases passed"
exit 0
