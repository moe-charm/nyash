#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
prog="$root/apps/selfhost/vm/mini_vm_if_branch.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$(NYASH_ENABLE_USING=1 NYASH_VM_USE_PY=1 "$bin" --backend vm "$prog" 2>/dev/null)
test "$out" = "10" || { echo "[FAIL] mini_vm_if_branch expected 10, got '$out'" >&2; exit 2; }
echo "[OK] mini_vm_if_branch"
exit 0
