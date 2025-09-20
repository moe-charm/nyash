#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/tests/strings/utf8_cp_demo.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_VM_USE_PY=1
unset NYASH_MINIVM_READ_STDIN || true
out=$("$bin" --backend vm "$src" 2>/dev/null)
want=$(printf "3\n1\n1\né𝄞\n")
test "$out" = "$want" || { echo "[FAIL] utf8 cp smoke: expected\\n$want\\ngot\\n$out" >&2; exit 2; }
echo "[OK] utf8 cp smoke"
