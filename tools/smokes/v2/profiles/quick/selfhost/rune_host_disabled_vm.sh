#!/bin/bash
# rune_host_disabled_vm.sh — RuneHost skeleton is disabled by default

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/rune_host_disabled_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "apps/selfhost/vm/boxes/rune_host.hako" as RuneHostBox

static box Main {
  main() {
    local ctx = new MapBox()
    local rc = RuneHostBox.eval("1+2", ctx)
    if rc < 0 { print("disabled") } else { print("ok:" + (""+rc)) }
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '\r')
expected='disabled'
compare_outputs "$expected" "$out" "rune_host_disabled_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
