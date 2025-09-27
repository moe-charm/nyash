#!/bin/bash
# userbox_using_package_vm.sh — Using alias/package → prelude AST → new + method (prod)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USING_PROFILE=prod
export NYASH_USING_AST=1
export NYASH_VM_USER_INSTANCE_BOXCALL=0

TEST_DIR="/tmp/userbox_using_package_vm_$$"
mkdir -p "$TEST_DIR" && cd "$TEST_DIR"

cat > nyash.toml << 'EOF'
[using.lib_pkg]
path = "."
main = "lib.nyash"

[using.aliases]
Lib = "lib_pkg"
EOF

cat > lib.nyash << 'EOF'
box LibBox {
  value() { return 5 }
}
EOF

cat > driver.nyash << 'EOF'
using Lib as Lib

static box Main {
  main() {
    local o = new LibBox()
    if o.value() == 5 { print("ok") } else { print("ng") }
    return 0
  }
}
EOF

output=$(run_nyash_vm driver.nyash --dev)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)
[ "$output" = "ok" ] && test_pass "userbox_using_package_vm" || test_fail "userbox_using_package_vm" "got: $output"

cd /
rm -rf "$TEST_DIR"

