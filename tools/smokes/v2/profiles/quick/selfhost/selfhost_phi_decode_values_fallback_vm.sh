#!/bin/bash
# selfhost_phi_decode_values_fallback_vm.sh — PhiDecodeBox values[] fallback smoke (no pred match -> first value)

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Run only when explicitly enabled; keep quick suite lean
if [ "${SMOKES_ENABLE_PHI_DECODE_EXTRA:-0}" != "1" ]; then
  echo "[SKIP] extra phi decode (fallback)" >&2
  exit 0
fi
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

TMP_DIR="/tmp/selfhost_phi_decode_values_fallback_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.boxes.phi_decode_box as PhiDecodeBox

static box Main {
  main() {
    // values form without matching pred; expect fallback to first value (1)
    local seg = "{\"op\":\"phi\",\"dst\":3,\"values\":[{\"pred\":5,\"value\":1},{\"pred\":6,\"value\":2}]}"
    local res = PhiDecodeBox.decode_result(seg, 9)
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
compare_outputs "$expected" "$out" "selfhost_phi_decode_values_fallback_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

