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
{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Int","value":0}}]}
JSON

set +e
out=$(NYASH_CLI_VERBOSE=1 "$BIN" --backend vm --json-file "$JSON" --using "no.such.ns as X" 2>&1)
rc=$?
set -e

echo "$out" | rg -q "\[using\] unresolved 'no\.such\.ns'" || { echo "FAIL: unresolved hint not shown" >&2; echo "$out" >&2; exit 1; }
echo "$out" | rg -q '^Result:\s*0\b' || { echo "FAIL: execution result not 0" >&2; echo "$out" >&2; exit 1; }
echo "PASS: using unresolved hint (CLI)" >&2
echo "All PASS" >&2
