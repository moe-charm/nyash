#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/selfhost-vm/mini_vm.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_VM_USE_PY=1
export NYASH_MINIVM_READ_STDIN=1

json='{"kind":"Program","statements":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"hello"}}},{"kind":"Print","expression":{"kind":"Literal","value":{"type":"int","value":123}}}]}'
out=$(printf '%s' "$json" | "$bin" --backend vm "$src" 2>&1)
echo "$out" | rg -q '^hello$' || { echo "[FAIL] line1 not hello" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q '^123$' || { echo "[FAIL] line2 not 123" >&2; echo "$out" >&2; exit 2; }
echo "[OK] mini-vm print multi literal smoke passed"

