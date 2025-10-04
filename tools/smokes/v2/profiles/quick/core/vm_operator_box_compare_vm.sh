#!/bin/bash
# vm_operator_box_compare_vm.sh — OperatorBox compare parity (debug box)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/vm_operator_box_compare_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "apps/selfhost/vm/boxes/operator_box.hako" as OperatorBox

static box Main {
  main() {
    print("EQ="+(""+OperatorBox.compare("Eq", 5, 5)))
    print("NE="+(""+OperatorBox.compare("Ne", 5, 7)))
    print("LT="+(""+OperatorBox.compare("Lt", 3, 4)))
    print("LE="+(""+OperatorBox.compare("Le", 3, 3)))
    print("GT="+(""+OperatorBox.compare("Gt", 9, 4)))
    print("GE="+(""+OperatorBox.compare("Ge", 9, 9)))
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '\r' | tail -n 6 | xargs echo)
expected="EQ=1 NE=1 LT=1 LE=1 GT=1 GE=1"

test_name="vm_operator_box_compare_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

