#!/bin/bash
# vm_llvm_equality_core.sh - VM↔LLVM parity for equality (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_parity_equality_core() {
  check_parity -c '
    box Point {
      x; y;
      birth(a,b){ me.x=a; me.y=b }
      equals(o){ return me.x == o.x and me.y == o.y }
    }
    box Ref { equals(o){ return me == o } }
    static box Main { main() {
      // 1) alias pointer equality
      local a = new Point(1,2); local b = a;
      if a == b { print("t1") } else { print("f1") }
      // 2) equals recursion guard (me == other inside equals)
      local r1 = new Ref(); local r2 = new Ref();
      if r1 == r1 { print("t2") } else { print("f2") }
      if r1 == r2 { print("f3") } else { print("t3") }
      // 3) user-defined equals true/false
      local p1 = new Point(3,4); local p2 = new Point(3,4); local p3 = new Point(3,5);
      if p1 == p2 { print("t4") } else { print("f4") }
      if p1 == p3 { print("f5") } else { print("t5") }
      // 4) primitives
      if 42 == 42 { print("t6") } else { print("f6") }
      if "hi" == "hi" { print("t7") } else { print("f7") }
      return 0
    }}
  ' "vm_llvm_equality_core"
}

run_test "vm_llvm_equality_core" test_vm_llvm_parity_equality_core

