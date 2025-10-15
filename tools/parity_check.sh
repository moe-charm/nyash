#!/usr/bin/env bash
set -euo pipefail

# Basic VM↔LLVM parity checker with noise filtering
# Usage:
#   tools/parity_check.sh file.nyash
#   tools/parity_check.sh -c 'code here'

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT/.."

# Load filters from smokes if available
if [ -f "$ROOT/smokes/v2/lib/test_runner.sh" ]; then
  # shellcheck disable=SC1090
  source "$ROOT/smokes/v2/lib/test_runner.sh"
fi

filter() {
  if type parity_filter_noise >/dev/null 2>&1; then
    parity_filter_noise
  elif type filter_noise >/dev/null 2>&1; then
    filter_noise
  else
    cat
  fi
}

if [ "${1:-}" = "-c" ]; then
  CODE="$2"
  VM_OUT=$(./target/release/nyash -c "$CODE" 2>&1 | filter)
  LLVM_OUT=$(NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm -c "$CODE" 2>&1 | filter)
else
  FILE="$1"
  VM_OUT=$(./target/release/nyash "$FILE" 2>&1 | filter)
  LLVM_OUT=$(NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm "$FILE" 2>&1 | filter)
fi

VM_N=$(echo "$VM_OUT" | sed 's/[[:space:]]*$//' | sort)
LL_N=$(echo "$LLVM_OUT" | sed 's/[[:space:]]*$//' | sort)

if [ "$VM_N" = "$LL_N" ]; then
  echo "[parity] OK"
  exit 0
else
  echo "[parity] MISMATCH" >&2
  echo "--- VM ---" >&2
  echo "$VM_OUT" >&2
  echo "--- LLVM ---" >&2
  echo "$LLVM_OUT" >&2
  exit 1
fi

