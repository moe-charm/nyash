#!/bin/bash
# builder_autobirth_cross_module_vm.sh — Builder auto-birth across modules (file-using)
# tags: core builder autobirth cross-module

# SKIP unless explicitly enabled (stabilization step)
source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_ALLOW_USING_FILE=1
# Force builder path; avoid VM fallback auto-birth
export NYASH_BUILDER_NEWBOX_AUTOBIRTH=1
export NYASH_VM_AUTO_BIRTH_CPP=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/builder_autobirth_cross_module_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/other.hako" << 'EOF_OTHER'
// Cross-module class with birth in different file
box Life {
  _name
  birth(name) { me._name = name print(name) return 0 }
  name() { return me._name }
}
EOF_OTHER

cat > "$TMP_DIR/main.nyash" << 'EOF_MAIN'
using "./other.hako" as Life

static box Main {
  main() {
    // Expect builder to emit Other.Life.birth/1 after New Other.Life("Alice")
    local alice = new Life("Alice")
    /* printed in birth */ return 0
    return 0
  }
}
EOF_MAIN

out=$(run_nyash_vm "$TMP_DIR/main.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="Alice"
compare_outputs "$expected" "$out" "builder_autobirth_cross_module_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
