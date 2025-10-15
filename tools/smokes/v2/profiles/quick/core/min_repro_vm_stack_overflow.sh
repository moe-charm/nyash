#!/bin/bash
# min_repro_vm_stack_overflow.sh — minimal driver to reproduce VM stack overflow when calling Mini-VM with compare→ret
source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/min_repro_vm_overflow_$$"
mkdir -p "$TMP_DIR"

cat > "/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // Multi-compare with final ret r6; previously triggers stack overflow
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":["
    j = j + "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":3}},"
    j = j + "{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":3}},"
    j = j + "{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2},"
    j = j + "{\"op\":\"compare\",\"dst\":4,\"operation\":\"!=\",\"lhs\":1,\"rhs\":2},"
    j = j + "{\"op\":\"compare\",\"dst\":5,\"cmp\":\"Lt\",\"lhs\":2,\"rhs\":1},"
    j = j + "{\"op\":\"const\",\"dst\":7,\"value\":{\"type\":\"i64\",\"value\":5}},"
    j = j + "{\"op\":\"const\",\"dst\":8,\"value\":{\"type\":\"i64\",\"value\":4}},"
    j = j + "{\"op\":\"compare\",\"dst\":6,\"cmp\":\"Gt\",\"lhs\":7,\"rhs\":8},"
    j = j + "{\"op\":\"ret\",\"value\":6}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF

# Variant A: with fallback birth (default)
out1=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev 2>&1 | tail -n 1 | tr -d "\r" | xargs || true)
echo "[A] output: $out1"

# Variant B: disable NewBox→birth fallback to test hypothesis
export NYASH_VM_BIRTH_AFTER_NEW=0
out2=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev 2>&1 | tail -n 1 | tr -d "\r" | xargs || true)
echo "[B] output: $out2"

rm -rf "$TMP_DIR"
exit 0
