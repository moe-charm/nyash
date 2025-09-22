#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=$(cd "$(dirname "$0")/../../../.." && pwd)

echo "[smoke] collect_prints using + mixed order ..." >&2

pushd "$ROOT_DIR" >/dev/null

cargo build --release -q

export NYASH_ENABLE_USING=1
export NYASH_VM_USE_PY=1
# Ensure JSON plugin is loaded (MiniVmPrints uses JsonDocBox/JsonNodeBox)
export NYASH_LOAD_NY_PLUGINS=1
# seam safety valve for inlining (default-OFF elsewhere)
export NYASH_RESOLVE_FIX_BRACES=1
# keep dedup OFF for stability (resolver dedup is dev-only)
unset NYASH_RESOLVE_DEDUP_BOX || true
unset NYASH_RESOLVE_DEDUP_FN || true
# parser seam guard (default-OFF): ensure 'static box' at top-level is not mistaken for initializer
export NYASH_PARSER_STATIC_INIT_STRICT=1
BIN=./target/release/nyash
APP=apps/selfhost/vm/collect_mixed_using_smoke.nyash

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

# Seam hygiene check: ensure prelude_brace_delta==0 on the dump
NYASH_PYVM_DUMP_CODE=1 NYASH_RESOLVE_SEAM_DEBUG=1 NYASH_RESOLVE_FIX_BRACES=1 NYASH_RESOLVE_DEDUP_BOX=1 \
  "$BIN" --backend vm "$APP" >/dev/null 2>&1 || true
INS_OUT=$("$BIN" --backend vm apps/tests/dev_seam_inspect_dump.nyash)
echo "$INS_OUT" | grep -q "^prelude_brace_delta=0$" || {
  echo "[smoke] FAIL: seam prelude_brace_delta is not zero" >&2
  echo "$INS_OUT" >&2
  exit 1
}
echo "[smoke] OK: seam prelude brace delta == 0" >&2
popd >/dev/null
