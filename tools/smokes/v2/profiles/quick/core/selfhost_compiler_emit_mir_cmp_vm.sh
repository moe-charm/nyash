#!/bin/bash
# selfhost_compiler_emit_mir_cmp_vm.sh — Selfhost compiler emits MIR for Compare and Mini‑VM prints 1
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_compiler_emit_mir_cmp_vm_$$"
mkdir -p "$TMP_DIR"

# 1) Run selfhost compiler wrapper to get MIR(JSON v0) for Return(Compare(5,4) Gt)
#    Use JSON-only mode to avoid runner summary lines mixing into capture
JSON=$(NYASH_JSON_ONLY=1 run_nyash_vm "$NYASH_ROOT/apps/dev/selfhost_compiler_min_cmp.nyash" --dev | tail -n 1)
# Minimal sanity: ensure JSON looks like MIR v0
if ! printf '%s' "$JSON" | grep -q '"functions"'; then
  echo "[sanity] expected MIR JSON, got: $(echo "$JSON" | head -c 120)" >&2
  exit 1
fi

# Escape JSON for embedding into Ny string
ESCAPED=$(printf '%s' "$JSON" | sed -e 's/\\/\\\\/g' -e 's/"/\\\"/g')

# 2) Build a tiny driver to feed the JSON to MirVmMin
cat > "$TMP_DIR/driver.nyash" << EOF
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    local j = "$ESCAPED"
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_compiler_emit_mir_cmp_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
