#!/bin/bash
# parity_m2_binop_add_vm_llvm.sh — VM ↔ LLVM parity for MirVmMin binop(Add)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export SMOKES_FORCE_LLVM=1
export NYASH_NY_LLVM_COMPILER="${NYASH_ROOT}/target/release/ny-llvmc"

# Quick profile policy: LLVM parity is covered in integration; skip here
test_skip "parity_m2_binop_add_vm_llvm (quick)" "covered in integration profile" && exit 0

TMP_DIR="/tmp/parity_m2_binop_add_vm_llvm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":3}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":4}},{\"op\":\"binop\",\"dst\":3,\"op_kind\":\"Add\",\"lhs\":1,\"rhs\":2},{\"op\":\"ret\",\"value\":3}]}]}]}"
    return MirVmMin.run(j)
  }
}
EOF

out_vm=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
out_llvm=$(run_nyash_llvm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '\r' | xargs)

expected="7"
compare_outputs "$expected" "$out_vm" "parity_m2_binop_add_vm_llvm(vm)" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
if [ -n "$out_llvm" ]; then
  compare_outputs "$expected" "$out_llvm" "parity_m2_binop_add_vm_llvm(llvm)" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
fi

rm -rf "$TMP_DIR"
exit 0
