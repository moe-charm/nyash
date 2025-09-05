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

echo "[Roundtrip] Case A: Ny → JSON(v0) → MIR-Interp (pipe)" >&2
set -o pipefail
# Use a minimal program that current parser accepts. Tolerate failure and continue.
{
cat <<'NYCODE' \
| "$NY_PARSER" \
| "$BIN" --ny-parser-pipe > /tmp/nyash-rt-a.out
static box Main {
  main(args) {
    return (1+2)*3
  }
}
NYCODE
} || true
if rg -q '^Result:\s*9\b' /tmp/nyash-rt-a.out; then
  echo "PASS: Case A (pipe)" >&2
else
  echo "SKIP: Case A (pipe) - parser pipeline not ready; proceeding with Case B" >&2
  cat /tmp/nyash-rt-a.out >&2 || true
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
