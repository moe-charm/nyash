#!/bin/bash
# plugin_array_min_vm.sh - Minimal ArrayBox smoke (quick)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_arraybox_min_ops() {
  # Prefer plugin path if available; builtin path also acceptable
  local script='
  local a, n, x
  a = new ArrayBox()
  a.push("foo")
  a.push("bar")
  n = a.len()
  x = a.get(1)
  print(n)
  print(x)
  '
  local output
  output=$(NYASH_VM_PLUGIN_PREFER_ARRAY=1 NYASH_CLI_VERBOSE=0 run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
  # Expect length then element "bar"
  local last2
  last2=$(echo "$output" | tail -n 2 | tr '\n' '|')
  if [[ "$last2" == *"2|bar"* ]]; then
    test_pass "arraybox_min_ops"
  else
    compare_outputs "2|bar" "$last2" "arraybox_min_ops"
  fi
}

run_test "arraybox_min_ops" test_arraybox_min_ops
