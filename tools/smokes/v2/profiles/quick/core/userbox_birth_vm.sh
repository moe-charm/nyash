#!/bin/bash
# userbox_birth_vm.sh — Verify user instance box birth with auto-birth and explicit birth(args)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_birth_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
box TreeNode {
  name: String
  value: Integer

  birth(n, v) {
    me.name = n
    me.value = v
    return me
  }

  get_name() { return me.name }
  get_value() { return me.value }
}

static box Main {
  main() {
    // 1) auto-birth: new TreeNode(args) should call birth(args)
    local a = new TreeNode("A", 10)
    print(a.get_name())
    print(a.get_value())

    // 2) explicit birth: new TreeNode() + birth(args)
    local b = new TreeNode()
    b.birth("B", 20)
    print(b.get_name())
    print(b.get_value())
    return 0
  }
}
EOF

output=$(run_nyash_vm "$SRC")

expected=$(cat << 'TXT'
A
10
B
20
TXT
)

compare_outputs "$expected" "$output" "userbox_birth_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0

