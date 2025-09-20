#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/tests/strings/byte_ascii_demo.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_VM_USE_PY=1
unset NYASH_MINIVM_READ_STDIN || true
out=$("$bin" --backend vm "$src" 2>/dev/null)
want=$(printf "15\n5\nworld\n")
test "$out" = "$want" || { echo "[FAIL] byte ascii smoke: expected\\n$want\\ngot\\n$out" >&2; exit 2; }
echo "[OK] byte ascii smoke"
