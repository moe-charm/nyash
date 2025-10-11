#!/bin/bash
# rune_host_mock_vm.sh — Placeholder smoke for RuneHost mock provider (SKIP until provider wired)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/rune_host_mock_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "selfhost/vm/boxes/rune_host.hako" as RuneHostBox

static box Main {
  main() {
    local ctx = new MapBox()
    // Expect mock to evaluate a simple arithmetic expression
    local rc = RuneHostBox.eval("1+2", ctx)
    if rc == 3 { print("ok:3") } else { print("ng:" + (""+rc)) }
    return 0
  }
}
NY

HAKO_RUNE_ENABLE=1 HAKO_RUNE_PROVIDER=mock out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '\r')
expected='ok:3'
compare_outputs "$expected" "$out" "rune_host_mock_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
