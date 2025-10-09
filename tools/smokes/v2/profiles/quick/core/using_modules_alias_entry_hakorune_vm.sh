#!/bin/bash
# using_modules_alias_entry_hakorune_vm.sh — Verify [modules] alias for hakorune.vm.entry resolves and runs
# tags: core, using, entry

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_entry_hakorune_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using hakorune.vm.entry as HakoruneVmEntryBox

static box Main {
  main() {
    // Minimal MIR v0: main(){ const 1; ret %1 }
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = HakoruneVmEntryBox.run_min(j)
    print(HakoruneVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF

out_full=$(run_nyash_vm "$SRC" --dev)
if echo "$out_full" | grep -qi 'AST prelude merge is disabled\|using: file paths are disallowed'; then
  log_warn "SKIP using_modules_alias_entry_hakorune_vm (using resolver disabled)"
  cd /; rm -rf "$TMP_DIR"; exit 0
fi
out=$(echo "$out_full" | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "using_modules_alias_entry_hakorune_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
