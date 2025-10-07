#!/bin/bash
# json_macro_nested_vm.sh — Nested json/map passthrough

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros:self"
export NYASH_SKIP_TOML_ENV=1
export NYASH_USING=0
export NYASH_MACRO_BOX_CHILD_RUNNER=0

test_json_macro_nested() {
  local code=$(cat << 'NYCODE'
static box Main {
  main() {
    local m = json({ nest: map({ x: 7 }), arr: arr([1,2,3]) })
    print(m.get("nest").get("x"))
    print(m.get("arr").get(2))
    return 0
  }
}
NYCODE
)
  NYASH_MACRO_SELFHOST_MIN=1 NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako NYASH_SYNTAX_SUGAR_LEVEL=full run_nyash_vm -c "$code" --dev > /tmp/_out.json_macro_nested 2>&1
  local rc=$?
  local out
  out=$(cat /tmp/_out.json_macro_nested | grep -v '^void$' | grep -v '^Result:')
  if [ $rc -ne 0 ]; then
    test_fail "json_macro_nested_vm" "exit=$rc"; return 1
  fi
  if echo "$out" | grep -q "Unresolved function\|Parse error"; then
    test_fail "json_macro_nested_vm" "resolver error"; return 1
  fi
  test_pass "json_macro_nested_vm"
}

run_test "json_macro_nested_vm" test_json_macro_nested
