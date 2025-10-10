#!/bin/bash
# string_find_replace_last_vm.sh — Plugins: String find/replace/lastIndexOf

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_string_find_replace_last_vm() {
  local code='static box Main { main() {
    local s = "abracadabra"
    print(s.indexOf("bra", 0))      // 1
    print(s.lastIndexOf("bra", 99)) // 8
    print(s.replace("abra", "AB"))  // ABcadABra
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code" --dev | grep -v '^Result:')
  local last3; last3=$(echo "$out" | tail -n 3 | tr '\n' '|')
  if [[ "$last3" == *"1|8|ABcadABra|"* ]]; then
    return 0
  else
    compare_outputs "1|8|ABcadABra|" "$last3" "string_find_replace_last_vm"
  fi
}

run_test "string_find_replace_last_vm" test_string_find_replace_last_vm
