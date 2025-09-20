#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
prog="$root/apps/selfhost-vm/mini_vm.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

# Minimal AST JSON with a single print of int literal 42
json='{"kind":"Program","statements":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"int","value":42}}}]}'
out=$(NYASH_VM_USE_PY=1 "$bin" --backend vm "$prog" -- "$json" 2>/dev/null)
test "$out" = "42" || { echo "[FAIL] mini_vm_print_literal expected 42, got '$out'" >&2; exit 2; }
echo "[OK] mini_vm_print_literal"
exit 0
