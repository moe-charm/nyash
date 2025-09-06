#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [ ! -x "$BIN" ]; then
  cargo build --release --features cranelift-jit >/dev/null
fi

TMPJSON=$(mktemp)
cat >"$TMPJSON" <<'JSON'
{"version":0,"kind":"Program","body":[
  {"type":"Extern","iface":"env.modules","method":"set","args":[{"type":"Str","value":"acme.mod"},{"type":"Int","value":42}]},
  {"type":"Return","expr":{"type":"Binary","op":"-","lhs":{"type":"Extern","iface":"env.modules","method":"get","args":[{"type":"Str","value":"acme.mod"}]},"rhs":{"type":"Int","value":42}}}
]}
JSON

NYASH_DISABLE_PLUGINS=1 "$BIN" --backend vm --json-file "$TMPJSON" > /tmp/nyash-modules-smoke.out || true
if rg -q '^Result:\s*0\b' /tmp/nyash-modules-smoke.out; then
  echo "PASS: modules (env.modules set/get)" >&2
else
  echo "FAIL: modules (env.modules set/get)" >&2
  sed -n '1,120p' /tmp/nyash-modules-smoke.out >&2
  exit 1
fi

echo "All PASS" >&2
