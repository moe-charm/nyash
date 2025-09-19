#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro_golden_identity.nyash"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros/examples/hang_macro.nyash"
export NYASH_NY_COMPILER_TIMEOUT_MS=200  # keep test quick
export NYASH_MACRO_STRICT=1              # strict should fail process

set +e
"$bin" --dump-expanded-ast-json "$src" >/dev/null 2>&1
code=$?
set -e

if [ $code -eq 0 ]; then
  echo "Expected failure on macro timeout in strict mode" >&2
  exit 2
fi

echo "[OK] macro timeout strict mode fails as expected (exit=$code)"
