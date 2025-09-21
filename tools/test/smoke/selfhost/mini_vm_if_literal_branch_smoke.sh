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

# cond=1 -> then prints "T" only
json_then='{"kind":"Program","statements":[{"kind":"If","condition":{"kind":"Literal","value":{"type":"int","value":1}},"then_body":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"T"}}}],"else_body":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"F"}}}]}]}'
out=$(printf '%s' "$json_then" | "$bin" --backend vm "$src" 2>&1)
echo "$out" | rg -qx 'T' || { echo "[FAIL] then branch did not print T only" >&2; echo "$out" >&2; exit 2; }

# cond=0 -> else prints "F" only
json_else='{"kind":"Program","statements":[{"kind":"If","condition":{"kind":"Literal","value":{"type":"int","value":0}},"then_body":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"T"}}}],"else_body":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"F"}}}]}]}'
out=$(printf '%s' "$json_else" | "$bin" --backend vm "$src" 2>&1)
echo "$out" | rg -qx 'F' || { echo "[FAIL] else branch did not print F only" >&2; echo "$out" >&2; exit 2; }

echo "[OK] mini-vm if literal branch smoke passed"
