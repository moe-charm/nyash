#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="$root/apps/tests/dev_sugar/at_local_basic.nyash"
tmp="${TMPDIR:-/tmp}/at_local_$$.nyash"
trap 'rm -f "$tmp"' EXIT

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

# Pre-expand and run
"$root/tools/dev/at_local_preexpand.sh" "$src" > "$tmp"
export NYASH_VM_USE_PY=1
out=$("$bin" --backend vm "$tmp" 2>/dev/null)
test "$out" = "1" || { echo "[FAIL] @ local preexpand expected 1, got '$out'" >&2; exit 2; }
echo "[OK] @ local preexpand smoke passed"

