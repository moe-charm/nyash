#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [ ! -x "$BIN" ]; then
  cargo build --release --features cranelift-jit >/dev/null
fi

JSON=$(mktemp)
cat >"$JSON" <<'JSON'
{"version":0,"kind":"Program","body":[
  {"type":"Return","expr":{"type":"Extern","iface":"env.modules","method":"get","args":[{"type":"Str","value":"Util"}]}}
]}
JSON

# Use direct path using via CLI
NYASH_DISABLE_PLUGINS=1 "$BIN" --backend vm --json-file "$JSON" \
  --using '"apps/selfhost-minimal/main.nyash" as Util' > /tmp/nyash-using-resolve.out

if rg -q '\.nyash' /tmp/nyash-using-resolve.out; then
  echo "PASS: using resolve (CLI direct path)" >&2
else
  echo "FAIL: using resolve (CLI direct path)" >&2
  sed -n '1,120p' /tmp/nyash-using-resolve.out >&2
  exit 1
fi

echo "All PASS" >&2

