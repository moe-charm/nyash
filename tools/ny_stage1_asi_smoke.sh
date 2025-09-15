#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  echo "[build] nyash (release) ..." >&2
  cargo build --release >/dev/null
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

echo "[Stage-1 ASI] return with semicolon" >&2
printf 'return 1+2*3;\n' >"$tmpdir/a1.ny"
"$BIN" --parser ny --backend vm "$tmpdir/a1.ny" >/tmp/asi1.out || true
rg -q '^Result:\s*7\b' /tmp/asi1.out && echo "PASS a1" >&2 || { echo "FAIL a1" >&2; cat /tmp/asi1.out >&2; exit 1; }

echo "[Stage-1 ASI] operator-continued newline" >&2
printf 'return 1+\n2*3\n' >"$tmpdir/a2.ny"
"$BIN" --parser ny --backend vm "$tmpdir/a2.ny" >/tmp/asi2.out || true
rg -q '^Result:\s*7\b' /tmp/asi2.out && echo "PASS a2" >&2 || { echo "FAIL a2" >&2; cat /tmp/asi2.out >&2; exit 1; }

echo "[Stage-1 ASI] return on next line + paren" >&2
printf 'return\n(1+2)*3;\n' >"$tmpdir/a3.ny"
"$BIN" --parser ny --backend vm "$tmpdir/a3.ny" >/tmp/asi3.out || true
rg -q '^Result:\s*9\b' /tmp/asi3.out && echo "PASS a3" >&2 || { echo "FAIL a3" >&2; cat /tmp/asi3.out >&2; exit 1; }

echo "[Stage-1 ASI] double semicolon tolerant" >&2
printf 'return 1+2*3;;\n' >"$tmpdir/a4.ny"
"$BIN" --parser ny --backend vm "$tmpdir/a4.ny" >/tmp/asi4.out || true
rg -q '^Result:\s*7\b' /tmp/asi4.out && echo "PASS a4" >&2 || { echo "FAIL a4" >&2; cat /tmp/asi4.out >&2; exit 1; }

echo "All Stage-1 ASI smokes PASS" >&2
