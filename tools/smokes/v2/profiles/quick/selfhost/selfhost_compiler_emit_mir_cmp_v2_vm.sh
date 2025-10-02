#!/bin/bash
# selfhost_compiler_emit_mir_cmp_v2_vm.sh — Selfhost compiler (pipeline v2) emits MIR(JSON) for Compare; run on Mini‑VM

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

# Pipeline V2 is experimental; run only when explicitly enabled
if [[ "${NYASH_PIPELINE_V2:-}" != "1" ]]; then
  test_skip "Pipeline V2 is experimental; set NYASH_PIPELINE_V2=1 to enable"
  exit 0
fi

TMP_DIR="/tmp/selfhost_compiler_emit_mir_cmp_v2_vm_$$"
mkdir -p "$TMP_DIR"

APP="$NYASH_ROOT/apps/dev/selfhost_compiler_min_cmp_v2.nyash"

# 1) Produce MIR(JSON) via selfhost compiler wrapper (stdout)
json=$(run_nyash_vm "$APP" --dev | tail -n 1)

# 2) Embed into a small driver and run with Mini‑VM
esc=$(printf '%s' "$json" | sed -e 's/\\/\\\\/g' -e 's/"/\\\"/g')
cat > "$TMP_DIR/driver.nyash" << EOF
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    local j = "$esc"
    return MirVmMin.run(j)
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_compiler_emit_mir_cmp_v2_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
