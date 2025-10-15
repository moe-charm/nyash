#!/bin/bash
# userbox_birth_reentrant_vm.sh — birth reentrancy must fail (in_birth guard)
# tags: core userbox contracts

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/userbox_birth_reentrant_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF_NY'
static box T {
  _v
  birth() {
    // Reentrant birth call: should be rejected by contracts (in_birth guard)
    me.birth()
    return 0
  }
}

static box Main {
  main() {
    local t = new T()
    print("ok") // should not reach here
    return 0
  }
}
EOF_NY

set +e
OUT=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev)
EC=$?
set -e
if [ $EC -ne 0 ]; then
  pass "userbox birth reentrancy fails as expected"
  rm -rf "$TMP_DIR"
  exit 0
else
  fail "userbox birth reentrancy unexpectedly succeeded"
  echo "$OUT" | filter_noise
  rm -rf "$TMP_DIR"
  exit 1
fi
