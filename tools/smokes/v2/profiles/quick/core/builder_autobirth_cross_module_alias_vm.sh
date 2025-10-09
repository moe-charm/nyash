#!/bin/bash
# builder_autobirth_cross_module_alias_vm.sh — Builder auto-birth across modules with different alias
# tags: core builder autobirth cross-module

# SKIP unless explicitly enabled (stabilization step)
source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_BUILDER_NEWBOX_AUTOBIRTH=1
export NYASH_VM_AUTO_BIRTH_CPP=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/builder_autobirth_cross_module_alias_vm_$$"
mkdir -p "$TMP_DIR/lib"

cat > "$TMP_DIR/lib/pet.hako" << 'EOF_PET'
box Life {
  _name
  birth(name) { me._name = name print(name) return 0 }
  name() { return me._name }
}
EOF_PET

cat > "$TMP_DIR/main.nyash" << 'EOF_MAIN'
using "./lib/pet.hako" as PetAlias

static box Main {
  main() {
    local alice = new Life("Mimi")
    /* printed in birth */ return 0
    return 0
  }
}
EOF_MAIN

out=$(run_nyash_vm "$TMP_DIR/main.nyash" --dev | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
expected="Mimi"
compare_outputs "$expected" "$out" "builder_autobirth_cross_module_alias_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
