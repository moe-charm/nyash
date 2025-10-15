#!/bin/bash
# vm_llvm_shortcircuit_core.sh - Short-circuit parity (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_shortcircuit_core() {
  check_parity -c '
    box Counter { v; birth(x){ me.v=x } inc(){ me.v = me.v + 1; return true } }
    static box Main { main() {
      local c = new Counter(0)
      // false and c.inc() → not evaluate right
      if false and c.inc() { print("bad") } else { print("ok1") }
      // true or c.inc() → not evaluate right
      if true or c.inc() { print("ok2") } else { print("bad") }
      // true and c.inc() → evaluate right
      if true and c.inc() { print("ok3") } else { print("bad") }
      // check counter value: should be exactly 1
      if c.v == 1 { print("ok4") } else { print("bad") }
      return 0
    }}
  ' "vm_llvm_shortcircuit_core"
}

run_test "vm_llvm_shortcircuit_core" test_vm_llvm_shortcircuit_core

