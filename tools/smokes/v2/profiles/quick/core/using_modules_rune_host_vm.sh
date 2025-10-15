#!/bin/bash
# using_modules_rune_host_vm.sh — Verify [modules] resolver for RuneHostBox

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1
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

out_full=$(run_nyash_vm "$TMP_DIR/driver.nyash")
if echo "$out_full" | grep -qi 'AST prelude merge is disabled\|using: file paths are disallowed'; then
  log_warn "SKIP using_modules_rune_host_vm (using resolver disabled in this env)"
  rm -rf "$TMP_DIR"; exit 0
fi
out=$(echo "$out_full" | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
expected='none'
compare_outputs "$expected" "$out" "using_modules_rune_host_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
