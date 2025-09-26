#!/usr/bin/env bash
# Archived: JIT smoke (not maintained in current phase). Kept for reference.
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../../" && pwd)

BIN="$ROOT_DIR/target/release/nyash"
if [ ! -x "$BIN" ]; then
  echo "Building nyash (release, JIT)..." >&2
  cargo build --release --features cranelift-jit >/dev/null
fi

echo "[JIT Smoke] Core VM/JIT (plugins disabled)" >&2
NYASH_DISABLE_PLUGINS=1 NYASH_CLI_VERBOSE=1 "$ROOT_DIR/tools/smokes/archive/smoke_vm_jit.sh" >/tmp/nyash-jit-core.out
grep -q '^✅ smoke done' /tmp/nyash-jit-core.out || { echo "FAIL: core VM/JIT smoke" >&2; cat /tmp/nyash-jit-core.out; exit 1; }
echo "PASS: core VM/JIT smoke" >&2

echo "All PASS (archived JIT smoke)" >&2

