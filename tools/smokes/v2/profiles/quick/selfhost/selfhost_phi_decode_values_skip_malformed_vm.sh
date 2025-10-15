#!/bin/bash
# selfhost_phi_decode_values_skip_malformed_vm.sh — skip malformed entry then use later valid entry

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Run only when explicitly enabled; keep quick suite lean
if [ "${SMOKES_ENABLE_PHI_DECODE_EXTRA:-0}" != "1" ]; then
  echo "[SKIP] extra phi decode (skip-malformed)" >&2
  exit 0
fi
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

TMP_DIR="/tmp/selfhost_phi_decode_values_skip_malformed_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using selfhost.vm.boxes.phi_decode_box as PhiDecodeBox

static box Main {
  main() {
    // First element malformed (no value), second valid with pred match
    local seg = "{\"op\":\"phi\",\"dst\":3,\"values\":[{\"pred\":0},{\"pred\":9,\"value\":7}]}"
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
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="dst=3,v=7"
compare_outputs "$expected" "$out" "selfhost_phi_decode_values_skip_malformed_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
