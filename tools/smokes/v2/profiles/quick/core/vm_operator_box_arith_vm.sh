#!/bin/bash
# vm_operator_box_arith_vm.sh — OperatorBox arithmetic parity (debug box)
# Gate: skip in quick profile by default (depends on operator box prelude availability)
if [ "${SMOKES_ENABLE_OPERATOR_BOX:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_OPERATOR_BOX=1" >&2
  exit 0
fi

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/vm_operator_box_arith_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "apps/selfhost/vm/boxes/operator_box.hako" as OperatorBox

static box Main {
  main() {
    print("ADD="+(""+OperatorBox.apply2("Add", 2, 6)))   // 8
    print("SUB="+(""+OperatorBox.apply2("Sub", 10, 7)))  // 3
    print("MUL="+(""+OperatorBox.apply2("Mul", 3, 6)))   // 18
    print("DIV="+(""+OperatorBox.apply2("Div", 8, 2)))   // 4
    print("MOD="+(""+OperatorBox.apply2("Mod", 9, 4)))   // 1
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '\r' | tail -n 5 | xargs echo)
expected="ADD=8 SUB=3 MUL=18 DIV=4 MOD=1"

test_name="vm_operator_box_arith_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

