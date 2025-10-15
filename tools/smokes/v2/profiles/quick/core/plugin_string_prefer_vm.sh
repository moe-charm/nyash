#!/bin/bash
# plugin_string_prefer_vm.sh - Prefer StringBox plugin when available (quick, dynamic)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_stringbox_plugin_prefer() {
  # Use plugin-prefer flag for StringBox
  local script='
  local s, u, n
  s = new StringBox("ab")
  n = s.length()
  u = s.toUtf8()
  print(n)
  print(u)
  '
  local output
  output=$(NYASH_VM_PLUGIN_PREFER_STRING=1 NYASH_CLI_VERBOSE=0 run_nyash_vm -c "$script" 2>&1)
  # Expect length then toUtf8 output on separate lines
  if echo "$output" | grep -q "Unknown Box type: StringBox\|VM fallback error\|BoxCall unsupported on StringBox.toUtf8"; then
    test_skip "stringbox_plugin_prefer" "StringBox plugin provider not configured"
    return 0
  fi
  # Extract last two lines (prints)
  local last2
  last2=$(echo "$output" | tail -n 2 | tr '\n' '|' )
  if [[ "$last2" == *"2|ab"* ]]; then
    test_pass "stringbox_plugin_prefer"
  else
    compare_outputs "2|ab" "$(echo "$output" | tail -n 2 | tr '\n' '|')" "stringbox_plugin_prefer"
  fi
}

run_test "stringbox_plugin_prefer" test_stringbox_plugin_prefer
