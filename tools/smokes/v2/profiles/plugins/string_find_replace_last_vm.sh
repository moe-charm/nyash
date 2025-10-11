#!/bin/bash
# string_find_replace_last_vm.sh — Plugins: String find/replace/lastIndexOf

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_string_find_replace_last_vm() {
  local code='static box Main { main() {
    local s = "abracadabra"
    if s.indexOf("bra", 0) == 1 { print("idx-ok") } else { print("idx-ng") }
    if s.lastIndexOf("bra", 99) == 8 { print("last-ok") } else { print("last-ng") }
    if s.replace("abra", "AB") == "ABcadAB" { print("rep-legacy") } else { print("rep-ng") }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code"  | grep -v '^Result:')
  local last3; last3=$(echo "$out" | tail -n 3 | tr '\n' '|')
  if [[ "$last3" == *"idx-ok|last-ok|rep-legacy|"* ]]; then
    return 0
  else
    compare_outputs "idx-ok|last-ok|rep-legacy|" "$last3" "string_find_replace_last_vm"
  fi
}

run_test "string_find_replace_last_vm" test_string_find_replace_last_vm
