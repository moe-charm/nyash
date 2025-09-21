#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../../.." && pwd)

echo "[smoke] collect_prints using + mixed order ..." >&2

pushd "$ROOT_DIR" >/dev/null

cargo build --release -q

export NYASH_ENABLE_USING=1
export NYASH_VM_USE_PY=1
# seam safety valve for inlining (default-OFF elsewhere)
export NYASH_RESOLVE_FIX_BRACES=1
# keep dedup OFF for stability (resolver dedup is dev-only)
unset NYASH_RESOLVE_DEDUP_BOX || true
unset NYASH_RESOLVE_DEDUP_FN || true
# parser seam guard (default-OFF): ensure 'static box' at top-level is not mistaken for initializer
export NYASH_PARSER_STATIC_INIT_STRICT=1
BIN=./target/release/nyash
APP=apps/selfhost-vm/collect_mixed_using_smoke.nyash

out=$("$BIN" --backend vm "$APP")

expected=$'A\nB\n7\n1\n7\n5'

if [[ "$out" != "$expected" ]]; then
  echo "[smoke] FAIL: unexpected output" >&2
  echo "--- got ---" >&2
  printf '%s\n' "$out" >&2
  echo "--- exp ---" >&2
  printf '%s\n' "$expected" >&2
  exit 1
fi

echo "[smoke] OK: collect_prints using + mixed order" >&2
popd >/dev/null
