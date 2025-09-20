#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro/strings/env_tag_demo.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/env_tag_string_macro.nyash"
# Do NOT set NYASH_MACRO_CAP_ENV; pass via ctx JSON instead
unset NYASH_MACRO_CAP_ENV || true

raw=$("$bin" --dump-expanded-ast-json "$src")
export NYASH_MACRO_CTX_JSON='{"caps":{"io":false,"net":false,"env":true}}'
out=$(printf '%s' "$raw" | "$bin" --macro-expand-child apps/macros/examples/env_tag_string_macro.nyash)
echo "$out" | rg -q '"value":"hello \[ENV\]"' && echo "[OK] env-tag macro applied via ctx JSON" && exit 0
echo "[FAIL] env-tag macro did not apply (ctx JSON path)" >&2
echo "$out" >&2
exit 2

