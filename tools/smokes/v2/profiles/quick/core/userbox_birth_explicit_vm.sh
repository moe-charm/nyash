#!/bin/bash
# userbox_birth_explicit_vm.sh — Verify explicit b.birth(args) initializes fields

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Gate: explicit dot-call birth() under refinement; opt-in for now.
if [ "${SMOKES_ENABLE_USERBOX_BIRTH:-0}" != "1" ]; then
  log_warn "SKIP userbox_birth_explicit_vm (set SMOKES_ENABLE_USERBOX_BIRTH=1 to run)"
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_birth_explicit_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
box Node {
  name: String
  value: Integer
  birth(n, v) { me.name = n me.value = v return me }
  get_name() { return me.name }
  get_value() { return me.value }
}

static box Main {
  main() {
    local b = new Node()
    b.birth("B", 20)
    print(b.get_name())
    print(b.get_value())
    return 0
  }
}
EOF

output=$(run_nyash_vm "$SRC")
expected=$(cat << 'TXT'
B
20
TXT
)
compare_outputs "$expected" "$output" "userbox_birth_explicit_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
