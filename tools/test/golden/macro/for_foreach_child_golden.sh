#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_BOX_CHILD=1

normalize() { python3 -c 'import sys,json; print(json.dumps(json.loads(sys.stdin.read()), sort_keys=True, separators=(",",":")))'; }

# for_: init=lambda(local i=0), cond, step=lambda(i=i+1), body=lambda(print(i))
in_for='{"kind":"Program","statements":[{"kind":"FunctionCall","name":"ny_for","arguments":[{"kind":"Lambda","params":[],"body":[{"kind":"Local","variables":["i"],"inits":[{"kind":"Literal","value":{"type":"int","value":0}}]}]},{"kind":"BinaryOp","op":"<","left":{"kind":"Variable","name":"i"},"right":{"kind":"Literal","value":{"type":"int","value":3}}},{"kind":"Lambda","params":[],"body":[{"kind":"Assignment","target":{"kind":"Variable","name":"i"},"value":{"kind":"BinaryOp","op":"+","left":{"kind":"Variable","name":"i"},"right":{"kind":"Literal","value":{"type":"int","value":1}}}}]},{"kind":"Lambda","params":[],"body":[{"kind":"Print","expression":{"kind":"Variable","name":"i"}}]}]}]}'
exp_for='{"kind":"Program","statements":[{"kind":"Local","variables":["i"],"inits":[{"kind":"Literal","value":{"type":"int","value":0}}]},{"kind":"Loop","condition":{"kind":"BinaryOp","op":"<","left":{"kind":"Variable","name":"i"},"right":{"kind":"Literal","value":{"type":"int","value":3}}},"body":[{"kind":"Print","expression":{"kind":"Variable","name":"i"}},{"kind":"Assignment","target":{"kind":"Variable","name":"i"},"value":{"kind":"BinaryOp","op":"+","left":{"kind":"Variable","name":"i"},"right":{"kind":"Literal","value":{"type":"int","value":1}}}}]}]}'

out_for=$(printf '%s' "$in_for" | "$bin" --macro-expand-child apps/macros/examples/for_foreach_macro.nyash)
if [ "$(printf '%s' "$out_for" | normalize)" != "$(printf '%s' "$exp_for" | normalize)" ]; then
  echo "[FAIL] for_ child transform mismatch" >&2
  diff -u <(printf '%s' "$exp_for" | normalize) <(printf '%s' "$out_for" | normalize) || true
  exit 2
fi

# foreach_: arr [1,2,3], var "x", body=lambda(print(x)) => i-loop with get(i)
in_fe='{"kind":"Program","statements":[{"kind":"FunctionCall","name":"ny_foreach","arguments":[{"kind":"Array","elements":[{"kind":"Literal","value":{"type":"int","value":1}},{"kind":"Literal","value":{"type":"int","value":2}},{"kind":"Literal","value":{"type":"int","value":3}}]},{"kind":"Literal","value":{"type":"string","value":"x"}},{"kind":"Lambda","params":[],"body":[{"kind":"Print","expression":{"kind":"Variable","name":"x"}}]}]}]}'
out_fe=$(printf '%s' "$in_fe" | "$bin" --macro-expand-child apps/macros/examples/for_foreach_macro.nyash)
outn=$(printf '%s' "$out_fe" | normalize)
if ! echo "$outn" | rg -q '"kind":"Loop"'; then
  echo "[FAIL] foreach_ did not produce a Loop" >&2
  echo "$out_fe" >&2
  exit 3
fi

echo "[OK] for_/foreach_ child transform goldens passed"

