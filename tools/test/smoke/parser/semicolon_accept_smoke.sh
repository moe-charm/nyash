#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
prog="$root/apps/tests/parser/semicolon_basic.nyash"

out=$(NYASH_PARSER_ALLOW_SEMICOLON=1 NYASH_VM_USE_PY=1 "$bin" --backend vm "$prog")
expected=$'A\nB'
test "$out" = "$expected" || { echo "[FAIL] semicolon_accept expected '$expected', got '$out'" >&2; exit 2; }
echo "[OK] semicolon_accept"

