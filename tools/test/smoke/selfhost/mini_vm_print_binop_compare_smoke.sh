#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/selfhost/vm/mini_vm.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_ENABLE_USING=1
export NYASH_VM_USE_PY=1
export NYASH_MINIVM_READ_STDIN=1

# BinaryOp int + int → 46
json1='{"kind":"Program","statements":[{"kind":"Print","expression":{"kind":"BinaryOp","operator":"+","left":{"kind":"Literal","value":{"type":"int","value":12}},"right":{"kind":"Literal","value":{"type":"int","value":34}}}}]}'
out1=$(printf '%s' "$json1" | "$bin" --backend vm "$src" 2>&1)
echo "$out1" | rg -qx '46' || { echo "[FAIL] BinaryOp int+int failed" >&2; echo "$out1" >&2; exit 2; }

echo "[OK] mini-vm binop (int) smoke passed"
