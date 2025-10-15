#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../.." && pwd)

echo "[dev] seam inspect (using mixed)" >&2

pushd "$ROOT_DIR" >/dev/null

cargo build --release -q

export NYASH_USING=1
export NYASH_VM_USE_PY=1
export NYASH_PYVM_DUMP_CODE=1
# optional seam logs for the target run
export NYASH_RESOLVE_SEAM_DEBUG=1
# optional safety/normalization for the target run (only needed to produce dump)
export NYASH_RESOLVE_FIX_BRACES=1
export NYASH_RESOLVE_DEDUP_BOX=1

BIN=./target/release/nyash
APP_MIX=apps/selfhost/vm/collect_mixed_using_smoke.nyash
APP_INS=apps/tests/dev_seam_inspect_dump.nyash

echo "[dev] run using-mixed app to produce dump ..." >&2
"$BIN" --backend vm "$APP_MIX" >/dev/null 2>&1 || true

echo "[dev] inspect dump ..." >&2
# prevent inspector run from overwriting dump and disable seam modifiers for inspector itself
unset NYASH_PYVM_DUMP_CODE
unset NYASH_RESOLVE_SEAM_DEBUG
unset NYASH_RESOLVE_FIX_BRACES
unset NYASH_RESOLVE_DEDUP_BOX

# Run inspector and capture output
OUT=$("$BIN" --backend vm "$APP_INS")
echo "$OUT"

# CI guard: brace delta must be zero after FIX_BRACES normalization in the dump
echo "[dev] assert: prelude_brace_delta==0" >&2
echo "$OUT" | grep -q "^prelude_brace_delta=0$" || {
  echo "[error] prelude_brace_delta is not zero" >&2
  exit 1
}

popd >/dev/null
