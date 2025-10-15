#!/bin/bash
# multi_branch_phi.sh - else-if 多分岐での PHI 配線（5枝）

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_multi_branch_phi() {
  local script='
local n, s
n = 3
if n == 1 {
  s = "one"
} else if n == 2 {
  s = "two"
} else if n == 3 {
  s = "three"
} else if n == 4 {
  s = "four"
} else {
  s = "many"
}
print(s)
'
  local output
  output=$(run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
  check_exact "three" "$output" "multi_branch_phi"
}

run_test "multi_branch_phi" test_multi_branch_phi
