#!/bin/bash
# using_modules_alias_entry_selfhost_vm.sh — Verify [modules] alias for selfhost.vm.entry resolves and runs
# tags: core, using, entry

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_entry_selfhost_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":2}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = MiniVmEntryBox.run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF

out_full=$(run_nyash_vm "$SRC" --dev)
if echo "$out_full" | grep -qi 'AST prelude merge is disabled\|using: file paths are disallowed'; then
  log_warn "SKIP using_modules_alias_entry_selfhost_vm (using resolver disabled)"
  cd /; rm -rf "$TMP_DIR"; exit 0
fi
out=$(echo "$out_full" | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
expected="2"
compare_outputs "$expected" "$out" "using_modules_alias_entry_selfhost_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
