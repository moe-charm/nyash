#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/selfhost/vm/collect_literal_eval.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_ENABLE_USING=1
export NYASH_VM_USE_PY=1
out=$("$bin" --backend vm "$src" 2>/dev/null)
test "$out" = "42" || { echo "[FAIL] collect_prints expected 42, got '$out'" >&2; exit 2; }
echo "[OK] mini-vm collect_prints(literal) smoke passed"
