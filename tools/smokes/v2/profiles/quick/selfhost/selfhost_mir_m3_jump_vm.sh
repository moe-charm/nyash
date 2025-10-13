#!/bin/bash
# selfhost_mir_m3_jump_vm.sh — jump(target) changes block
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
if [ "${SMOKES_SELFHOST_M2M3_ENABLE:-0}" != "1" ]; then test_skip "selfhost M2/M3 gated (set SMOKES_SELFHOST_M2M3_ENABLE=1)"; exit 0; fi

# Enabled: Mini‑VM branch/jump basic

# Dev-time guards
export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_BUILDER_REWRITE_INSTANCE=1

TMP_DIR="/tmp/selfhost_mir_m3_jump_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // block0: const 1 -> dst1; jump 2
    // block1: ret 9 (should not execute)
    // block2: ret 1
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":2}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"ret\",\"value\":9}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = MiniVmEntryBox.run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_mir_m3_jump_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
