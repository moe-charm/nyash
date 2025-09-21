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
  {"kind":"Print","expression":{"kind":"FunctionCall","name":"echo","arguments":[{"kind":"Literal","value":{"type":"string","value":"hello"}}]}},
  {"kind":"Print","expression":{"kind":"FunctionCall","name":"itoa","arguments":[{"kind":"Literal","value":{"type":"int","value":7}}]}},
  {"kind":"Print","expression":{"kind":"Compare","operation":"<","lhs":{"kind":"Literal","value":{"type":"int","value":1}},"rhs":{"kind":"Literal","value":{"type":"int","value":2}}}},
  {"kind":"Print","expression":{"kind":"BinaryOp","operator":"+","left":{"kind":"Literal","value":{"type":"int","value":3}},"right":{"kind":"Literal","value":{"type":"int","value":4}}}}
]}'
out=$(printf '%s' "$json" | "$bin" --backend vm "$src" 2>&1)

echo "$out" | sed -n '1p' | rg -qx 'hello' || { echo "[FAIL] line1 not hello" >&2; echo "$out" >&2; exit 2; }
echo "$out" | sed -n '2p' | rg -qx '7'     || { echo "[FAIL] line2 not 7" >&2; echo "$out" >&2; exit 2; }
echo "$out" | sed -n '3p' | rg -qx '1'     || { echo "[FAIL] line3 not 1 (compare)" >&2; echo "$out" >&2; exit 2; }
echo "$out" | sed -n '4p' | rg -qx '7'     || { echo "[FAIL] line4 not 7 (binop)" >&2; echo "$out" >&2; exit 2; }
echo "[OK] mini-vm print mixed (echo/itoa/compare/binop) smoke passed"
