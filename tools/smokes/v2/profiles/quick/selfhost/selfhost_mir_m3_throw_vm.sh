#!/bin/bash
# selfhost_mir_m3_throw_vm.sh — Ensure throw terminator returns distinct error code

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
if [ "${SMOKES_SELFHOST_M2M3_ENABLE:-0}" != "1" ]; then test_skip "selfhost M2/M3 gated (set SMOKES_SELFHOST_M2M3_ENABLE=1)"; exit 0; fi

TMP_DIR="/tmp/selfhost_mir_m3_throw_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // bb0: throw
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[" +
      "{\"id\":0,\"instructions\":[{\"op\":\"throw\",\"value\":{\"type\":\"String\",\"value\":\"boom\"}}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="-2"
compare_outputs "$expected" "$out" "selfhost_mir_m3_throw_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
