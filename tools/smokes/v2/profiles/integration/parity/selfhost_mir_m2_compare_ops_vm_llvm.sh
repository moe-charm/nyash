#!/bin/bash
# selfhost_mir_m2_compare_ops_vm_llvm.sh — VM vs LLVM harness parity for minimal compare ops

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

if ! "$NYASH_BIN" --version 2>/dev/null | grep -q "features.*llvm"; then
  test_skip "LLVM backend not available in this build"; exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_compare_ops_vm_llvm_$$"
mkdir -p "$TMP_DIR"

ops=(Eq Ne Lt Le Gt Ge)

for op in "${ops[@]}"; do
  cat > "$TMP_DIR/driver.nyash" << EOF
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // const 5 -> r1; const 4 -> r2; compare(${op}) -> r3; ret r3
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j = j + "{\\\"op\\\":\\\"const\\\",\\\"dst\\\":1,\\\"value\\\":{\\\"type\\\":\\\"i64\\\",\\\"value\\\":5}},"
    j = j + "{\\\"op\\\":\\\"const\\\",\\\"dst\\\":2,\\\"value\\\":{\\\"type\\\":\\\"i64\\\",\\\"value\\\":4}},"
    j = j + "{\\\"op\\\":\\\"compare\\\",\\\"cmp\\\":\\\"${op}\\\",\\\"lhs\\\":1,\\\"rhs\\\":2,\\\"dst\\\":3},"
    j = j + "{\\\"op\\\":\\\"ret\\\",\\\"value\\\":3}] }]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF
  output_vm=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev)
  NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev)
  test_name="selfhost_mir_m2_compare_${op}_vm_llvm"
  compare_outputs "$output_vm" "$output_llvm" "$test_name" || { cd /; rm -rf "$TMP_DIR"; exit 1; }
done

rm -rf "$TMP_DIR"
exit 0

