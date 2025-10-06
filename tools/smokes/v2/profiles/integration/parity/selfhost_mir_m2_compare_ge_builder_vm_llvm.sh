#!/bin/bash
# selfhost_mir_m2_compare_ge_builder_vm_llvm.sh — VM vs LLVM parity for compare Ge using MirJsonBuilderMin

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Harness-first: rely on run_nyash_llvm() to decide availability

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_compare_ge_builder_vm_llvm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin

static box Main {
  main() {
    // const 42 -> r1; const 42 -> r2; compare(Ge) -> r3; ret r3
    local j = MirJsonBuilderMin.make()
      |> MirJsonBuilderMin.start_module()
      |> MirJsonBuilderMin.start_function("main")
      |> MirJsonBuilderMin.start_block(0)
      |> MirJsonBuilderMin.add_const(1, 42)
      |> MirJsonBuilderMin.add_const(2, 42)
      |> MirJsonBuilderMin.add_compare("Ge", 1, 2, 3)
      |> MirJsonBuilderMin.add_ret(3)
      |> MirJsonBuilderMin.end_all()
      |> MirJsonBuilderMin.to_string()
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

output_vm=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev)
NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev)
compare_outputs "$output_vm" "$output_llvm" "selfhost_mir_m2_compare_ge_builder_vm_llvm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
