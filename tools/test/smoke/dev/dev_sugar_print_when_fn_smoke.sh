#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
tmp="${TMPDIR:-/tmp}/devsugar_print_when_fn_$$.nyash"
trap 'rm -f "$tmp"' EXIT

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

"$root/tools/dev/dev_sugar_preexpand.sh" "$root/apps/tests/dev_sugar/print_when_fn.nyash" > "$tmp"
export NYASH_VM_USE_PY=1
out=$("$bin" --backend vm "$tmp" 2>/dev/null)
test "$out" = "42" || { echo "[FAIL] dev sugar print!/when/fn expected 42, got '$out'" >&2; exit 2; }
echo "[OK] dev sugar print!/when/fn smokes passed"

