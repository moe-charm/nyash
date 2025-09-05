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
// using/nyash.link E2E sample (placeholder)
static box Main {
  init { }
  main(args) {
    // When using/nyash.link is active, modules can be resolved here.
    // Placeholder just returns 0 for now.
    return 0
  }
}
NYCODE
fi

NYASH_DISABLE_PLUGINS=1 NYASH_ENABLE_USING=1 NYASH_CLI_VERBOSE=1 "$BIN" --backend vm "$APP" > /tmp/nyash-using-e2e.out
if rg -q '^Result:\s*0\b' /tmp/nyash-using-e2e.out; then
  echo "PASS: using/nyash.link E2E (placeholder)" >&2
else
  echo "FAIL: using/nyash.link E2E" >&2; sed -n '1,120p' /tmp/nyash-using-e2e.out; exit 1
fi

echo "All PASS" >&2
