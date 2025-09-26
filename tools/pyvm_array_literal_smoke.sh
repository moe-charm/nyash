#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")"/.. && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [[ ! -x "$BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build --release >/dev/null)
fi

run() {
  NYASH_VM_USE_PY=1 NYASH_SYNTAX_SUGAR_LEVEL=basic "$BIN" --backend vm "$ROOT_DIR/apps/tests/array_literal_basic.nyash" 2>&1
}

OUT=$(run || true)
echo "$OUT" | rg -q '^3$' && echo "✅ PyVM: array literal basic" || { echo "❌ PyVM: array literal basic"; echo "$OUT"; exit 1; }

echo "Array literal smoke PASS" >&2

