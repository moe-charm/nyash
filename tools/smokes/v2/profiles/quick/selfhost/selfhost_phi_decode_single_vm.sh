#!/bin/bash
# selfhost_phi_decode_single_vm.sh — PhiDecodeBox single-form decode smoke

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_phi_decode_single_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.boxes.phi_decode_box as PhiDecodeBox

static box Main {
  main() {
    // single-form: {"op":"phi","dst":3,"pred":0,"value":1}
    local seg = "{\"op\":\"phi\",\"dst\":3,\"pred\":0,\"value\":1}"
    local res = PhiDecodeBox.decode_result(seg, 0)
    if res.is_ok() == 1 {
      local p = res.value()
      print("dst=" + (""+p.get(0)) + ",v=" + (""+p.get(1)))
      return 0
    }
    print("ERR")
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="dst=3,v=1"
compare_outputs "$expected" "$out" "selfhost_phi_decode_single_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

