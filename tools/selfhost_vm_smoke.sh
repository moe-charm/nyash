#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
BIN="$ROOT_DIR/target/release/nyash"
APP="$ROOT_DIR/apps/selfhost-minimal/main.nyash"

if [ ! -x "$BIN" ]; then
  echo "[selfhost] building nyash (release, JIT)..." >&2
  (cd "$ROOT_DIR" && cargo build --release --features cranelift-jit >/dev/null)
fi

if [ ! -f "$APP" ]; then
  echo "[selfhost] sample missing: $APP" >&2
  exit 2
fi

NYASH_DISABLE_PLUGINS=1 NYASH_CLI_VERBOSE=1 "$BIN" --backend vm "$APP" > /tmp/nyash-selfhost-minimal.out
if rg -q '^Result:\s*0\b' /tmp/nyash-selfhost-minimal.out; then
  echo "PASS: selfhost-minimal (VM path)" >&2
else
  echo "FAIL: selfhost-minimal" >&2
  sed -n '1,120p' /tmp/nyash-selfhost-minimal.out
  exit 1
fi

echo "All PASS" >&2

