#!/bin/bash
# selfhost_utils_guard_box_vm.sh — GuardBox basic tick

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_utils_guard_box_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.boxes.guard_box as GuardBox

static box Main {
  main() {
    local g = new GuardBox("g", 3)
    local i = 0
    loop(i < 3) {
      if g.tick() == 0 { print("ERR") return 0 }
      i = i + 1
    }
    print("OK")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="OK"
compare_outputs "$expected" "$out" "selfhost_utils_guard_box_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

