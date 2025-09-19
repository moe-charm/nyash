#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")"/../../../.. && pwd)
bin="$root/target/release/nyash"
src="apps/tests/macro_golden_identity.nyash"
golden="$root/tools/test/golden/macro/identity.expanded.json"

if [ ! -x "$bin" ]; then
  echo "nyash binary not found at $bin; build first (cargo build --release)" >&2
  exit 1
fi

out=$("$bin" --dump-expanded-ast-json "$src")

# Strip whitespace for robust compare
norm() { tr -d '\n\r\t ' <<< "$1"; }

if [ "$(norm "$out")" != "$(norm "$(cat "$golden")")" ]; then
  echo "Golden mismatch" >&2
  diff -u <(echo "$out") "$golden" || true
  exit 2
fi

echo "[OK] golden macro identity matched"
