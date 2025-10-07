#!/bin/bash
# call_macro_strict_vm.sh — call("Box.method/N", args...) normalization

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS="apps/macros:self"
export NYASH_SKIP_TOML_ENV=1
export NYASH_USING=0
export NYASH_MACRO_BOX_CHILD_RUNNER=0

test_call_macro_strict() {
  # Always enabled now that builder supports external ModuleFunction resolution
  local code=$(cat << 'NYCODE'
static box Main {
  main() {
    local s = "abc"
    // Normalize to FunctionCall name="String.len/1" arguments:[s]
    local n = call("String.len/1", s)
    print(n)
    return 0
  }
}
NYCODE
)
  NYASH_MACRO_SELFHOST_MIN=1 NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako NYASH_SYNTAX_SUGAR_LEVEL=full run_nyash_vm -c "$code" --dev > /tmp/_out.call_macro_strict 2>&1
  local rc=$?
  local out
  out=$(cat /tmp/_out.call_macro_strict | grep -v '^void$' | grep -v '^Result:')
  if [ $rc -ne 0 ]; then
    test_fail "call_macro_strict_vm" "exit=$rc"; return 1
  fi
  if echo "$out" | grep -q "Unresolved function\|Parse error"; then
    test_fail "call_macro_strict_vm" "resolver error"; return 1
  fi
  test_pass "call_macro_strict_vm"
}

run_test "call_macro_strict_vm" test_call_macro_strict
