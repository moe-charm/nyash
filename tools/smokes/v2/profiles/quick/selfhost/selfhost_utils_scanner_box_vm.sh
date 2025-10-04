#!/bin/bash
# selfhost_utils_scanner_box_vm.sh — ScannerBox basic scan

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_utils_scanner_box_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.boxes.scanner_box as ScannerBox

static box Main {
  main() {
    local s = new ScannerBox("ab")
    local out = ""
    loop(s.at_end() == 0) {
      local ch = s.advance()
      if ch == null { break }
      out = out + ch
    }
    print(out)
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ab"
compare_outputs "$expected" "$out" "selfhost_utils_scanner_box_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

