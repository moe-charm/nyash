#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"

if [ ! -x "$BIN" ]; then
  echo "[using-e2e] building nyash (release, JIT)..." >&2
  cargo build --release --features cranelift-jit >/dev/null
fi

APP="$ROOT_DIR/apps/using-e2e/main.nyash"
if [ ! -f "$APP" ]; then
  echo "[using-e2e] scaffolding sample..." >&2
  mkdir -p "$ROOT_DIR/apps/using-e2e"
  cat > "$APP" <<'NYCODE'
// using/nyash.link E2E sample (MVP)
using acme.util

static box Main {
  init { }
  main(args) {
    // using line should be accepted when NYASH_USING=1
    return 0
  }
}
NYCODE
fi

set +e
NYASH_DISABLE_PLUGINS=1 NYASH_USING=1 NYASH_CLI_VERBOSE=1 "$BIN" --backend vm "$APP" > /tmp/nyash-using-e2e.out
CODE=$?
set -e
# Accept either explicit "Result: 0" line (VM path) or zero exit (PyVM-only path)
if rg -q '^Result:\s*0\b' /tmp/nyash-using-e2e.out || [ "$CODE" -eq 0 ]; then
  echo "PASS: using/nyash.link E2E (placeholder)" >&2
else
  echo "FAIL: using/nyash.link E2E" >&2; sed -n '1,120p' /tmp/nyash-using-e2e.out; exit 1
fi

echo "All PASS" >&2
