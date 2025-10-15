#!/bin/bash
# selfhost_compiler_emit_mir_binop_cmp_vm_llvm.sh — Selfhost compiler emits MIR(JSON) for BinOp/Compare; VM vs LLVM parity

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

if ! "$NYASH_BIN" --version 2>/dev/null | grep -q "features.*llvm"; then
  test_skip "LLVM backend not available in this build"; exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_compiler_emit_mir_binop_cmp_vm_llvm_$$"
mkdir -p "$TMP_DIR"

emit_and_embed() {
  local mode=$1  # binop|cmp
  local app
  if [[ "$mode" == "binop" ]]; then
    app="$NYASH_ROOT/apps/dev/selfhost_compiler_min_binop.nyash"
  else
    app="$NYASH_ROOT/apps/dev/selfhost_compiler_min_cmp.nyash"
  fi
  # 1) produce MIR(JSON) via selfhost compiler wrapper
  local json
  json=$(run_nyash_vm "$app" --dev | tail -n 1)
  # 2) embed to a small driver
  local esc
  esc=$(printf '%s' "$json" | sed -e 's/\\/\\\\/g' -e 's/"/\\\"/g')
  cat > "$TMP_DIR/driver_${mode}.nyash" << EOF
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    local j = "$esc"
    return MirVmMin.run(j)
  }
}
EOF
}

for mode in binop cmp; do
  emit_and_embed "$mode"
  output_vm=$(run_nyash_vm "$TMP_DIR/driver_${mode}.nyash" --dev)
  NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$TMP_DIR/driver_${mode}.nyash" --dev)
  compare_outputs "$output_vm" "$output_llvm" "selfhost_compiler_emit_mir_${mode}_vm_llvm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
done

rm -rf "$TMP_DIR"
exit 0

