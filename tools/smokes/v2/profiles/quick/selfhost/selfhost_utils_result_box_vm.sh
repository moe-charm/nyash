#!/bin/bash
# selfhost_utils_result_box_vm.sh — ResultBox basic ok/err usage

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_utils_result_box_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.boxes.result_box as Result

static box Main {
  main() {
    local a = Result.ok(7)
    local b = Result.err("oops")
    local s = a.unwrap_or(0) + b.unwrap_or(5)
    print(""+s)
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="12"
compare_outputs "$expected" "$out" "selfhost_utils_result_box_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

