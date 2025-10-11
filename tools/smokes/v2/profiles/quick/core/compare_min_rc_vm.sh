#!/bin/bash
# compare_min_rc_vm.sh — minimal compare semantics (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

# numeric and string compares ok; BoxRef ordered compares not required in quick
code='
static box Main {
  main() {
    if 2 < 3 { } else { return 10 }
    if "a" < "b" { } else { return 11 }
    return 0
  }
}
'
if run_nyash_vm -c "$code" --dev >/dev/null; then
  test_pass "compare_min_rc_vm"
else
  test_fail "compare_min_rc_vm" "non-zero rc"
  exit 1
fi
