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

json='{"kind":"Program","statements":[
  {"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"a"}}},
  {"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"b"}}},
  {"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"c"}}},
  {"kind":"Print","expression":{"kind":"Literal","value":{"type":"int","value":1}}},
  {"kind":"Print","expression":{"kind":"Literal","value":{"type":"int","value":2}}}
]}'

out=$(printf '%s' "$json" | "$bin" --backend vm "$src" 2>&1)
echo "$out" | rg -q '^a$' || { echo "[FAIL] seq line1 not a" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q '^b$' || { echo "[FAIL] seq line2 not b" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q '^c$' || { echo "[FAIL] seq line3 not c" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q '^1$' || { echo "[FAIL] seq line4 not 1" >&2; echo "$out" >&2; exit 2; }
echo "$out" | rg -q '^2$' || { echo "[FAIL] seq line5 not 2" >&2; echo "$out" >&2; exit 2; }
echo "[OK] mini-vm print sequence (a,b,c,1,2) smoke passed"
