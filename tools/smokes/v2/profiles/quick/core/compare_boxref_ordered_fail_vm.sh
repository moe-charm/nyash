#!/bin/bash
# compare_boxref_ordered_fail_vm.sh — BoxRef ordered compare should fail (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

code='
static box Main {
  main() {
    // Ordered compare between two BoxRef values must error
    local a = new ArrayBox()
    local b = new ArrayBox()
    if a < b { return 99 }
    return 0
  }
}
'
# We expect non-zero exit due to TypeError at compare boundary
if run_nyash_vm -c "$code" --dev >/dev/null 2>&1; then
  test_fail "compare_boxref_ordered_fail_vm" "expected non-zero rc"
  exit 1
else
  test_pass "compare_boxref_ordered_fail_vm"
  exit 0
fi
