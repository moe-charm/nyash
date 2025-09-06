#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

BIN="$ROOT_DIR/target/release/nyash"
if [ ! -x "$BIN" ]; then
  echo "Building nyash (release)..." >&2
  cargo build --release --features cranelift-jit >/dev/null
fi

echo "[Smoke] Parser v0 JSON pipe → MIR-Interp" >&2
set -o pipefail
printf '{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}' \
  | "$BIN" --ny-parser-pipe >/tmp/nyash-bridge-smoke.out

if grep -q 'Result:' /tmp/nyash-bridge-smoke.out; then
  echo "PASS: pipe path" >&2
else
  echo "FAIL: pipe path" >&2; cat /tmp/nyash-bridge-smoke.out; exit 1
fi

echo "[Smoke] --json-file path" >&2
TMPJSON=$(mktemp)
cat >"$TMPJSON" <<'JSON'
{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}
JSON
"$BIN" --json-file "$TMPJSON" >/tmp/nyash-bridge-smoke2.out
if grep -q 'Result:' /tmp/nyash-bridge-smoke2.out; then
  echo "PASS: json-file path" >&2
else
  echo "FAIL: json-file path" >&2; cat /tmp/nyash-bridge-smoke2.out; exit 1
fi
echo "All PASS" >&2
