#!/bin/bash
# selfhost_phi_decode_values_all_malformed_vm.sh — values[] present but all elements malformed (no value) → ERR

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Run only when explicitly enabled; keep quick suite lean
if [ "${SMOKES_ENABLE_PHI_DECODE_EXTRA:-0}" != "1" ]; then
  echo "[SKIP] extra phi decode (all-malformed)" >&2
  exit 0
fi
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/selfhost_phi_decode_values_all_malformed_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using selfhost.vm.boxes.phi_decode_box as PhiDecodeBox

static box Main {
  main() {
    // values-form with two entries, both missing 'value' → should error with no-values
    local seg = "{\"op\":\"phi\",\"dst\":3,\"values\":[{\"pred\":0},{\"pred\":1}]}"
    local res = PhiDecodeBox.decode_result(seg, 0)
    if res.is_ok() == 1 {
      local p = res.value()
      print("dst=" + (""+p.get(0)) + ",v=" + (""+p.get(1)))
    } else {
      // Print error tag for visibility; Result.err(msg) is internal, Mini-VM prints nothing
      print("ERR")
    }
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ERR"
compare_outputs "$expected" "$out" "selfhost_phi_decode_values_all_malformed_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
