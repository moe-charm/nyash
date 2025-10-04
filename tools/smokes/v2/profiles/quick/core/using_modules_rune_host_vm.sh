#!/bin/bash
# using_modules_rune_host_vm.sh — Verify [modules] resolver for RuneHostBox

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_rune_host_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using selfhost.vm.rune_host as RuneHostBox

static box Main {
  main() {
    print(RuneHostBox.provider_name())
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '\r' | xargs)
expected='none'
compare_outputs "$expected" "$out" "using_modules_rune_host_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
