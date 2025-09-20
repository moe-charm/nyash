#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
prog="$root/apps/tests/pyvm/argv_echo.nyash"

out=$(NYASH_VM_USE_PY=1 "$bin" --backend vm "$prog" -- hello 2>/dev/null)
test "$out" = "hello" || { echo "[FAIL] pyvm argv inject expected 'hello', got '$out'" >&2; exit 2; }
echo "[OK] pyvm argv inject"

