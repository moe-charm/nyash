#!/usr/bin/env bash
set -euo pipefail
[[ "${NYASH_CLI_VERBOSE:-0}" == "1" ]] && set -x

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [ ! -x "$BIN" ]; then
  cargo build --release >/dev/null
fi

SRC=$(mktemp)
cat >"$SRC" <<'NY'
using ArrayBox
static box Main { main(args) { return 0 } }
NY

set +e
NYASH_USING=1 NYASH_PLUGIN_REQUIRE_PREFIX=1 "$BIN" --backend interpreter "$SRC" >/tmp/nyash-using-prefix-strict.out 2>&1
rc=$?
set -e

if [ $rc -ne 0 ] && rg -q "plugin short name 'ArrayBox' requires prefix" /tmp/nyash-using-prefix-strict.out; then
  echo "PASS: plugin short name rejected in strict mode" >&2
else
  echo "FAIL: strict plugin prefix not enforced" >&2
  sed -n '1,120p' /tmp/nyash-using-prefix-strict.out >&2 || true
  exit 1
fi

echo "All PASS" >&2
