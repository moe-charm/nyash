#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

BIN="$ROOT_DIR/target/release/nyash"
NY_PARSER="$ROOT_DIR/tools/ny_parser_run.sh"

if [ ! -x "$BIN" ]; then
  echo "Building nyash (release)..." >&2
  cargo build --release --features cranelift-jit >/dev/null
fi

echo "[Roundtrip] Case A: Ny (source v0) → MIR-Interp (direct bridge)" >&2
TMPNY=$(mktemp)
printf 'return 1+2*3\n' > "$TMPNY"
NYASH_DISABLE_PLUGINS=1 "$BIN" --parser ny --backend vm "$TMPNY" > /tmp/nyash-rt-a.out || true
if rg -q '^Result:\s*7\b' /tmp/nyash-rt-a.out; then
  echo "PASS: Case A (direct bridge)" >&2
else
  echo "FAIL: Case A (direct bridge)" >&2
  cat /tmp/nyash-rt-a.out >&2 || true
  exit 1
fi

echo "[Roundtrip] Case B: JSON(v0) file → MIR-Interp" >&2
TMPJSON=$(mktemp)
cat >"$TMPJSON" <<'JSON'
{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}
JSON
"$BIN" --json-file "$TMPJSON" > /tmp/nyash-rt-b.out
if rg -q '^Result:\s*7\b' /tmp/nyash-rt-b.out; then
  echo "PASS: Case B (json-file)" >&2
else
  echo "FAIL: Case B (json-file)" >&2; cat /tmp/nyash-rt-b.out; exit 1
fi

echo "All PASS" >&2
