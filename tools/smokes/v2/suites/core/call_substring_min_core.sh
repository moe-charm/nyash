#!/bin/bash
# call_substring_min_core.sh — Verify call!(String.substring/2) minimal (SKIP gated)

source "$(dirname "$0")/../../lib/test_runner.sh"

require_env || exit 2

if [ "${SMOKES_ENABLE_CORE_CALL_SUBSTR:-0}" != "1" ]; then
  test_skip "core call! substring/2 (set SMOKES_ENABLE_CORE_CALL_SUBSTR=1 to run)"; exit 0
fi

test_core_call_substring_min() {
  local code=$(cat << 'NY'
static box Main {
  main() {
    local s = "abc"
    local t = call("String.substring/2", s, 1, 2)
    print(t)
    return 0
  }
}
NY
)
  out=$(NYASH_MACRO_SELFHOST_MIN=1 NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako NYASH_SYNTAX_SUGAR_LEVEL=full run_nyash_vm -c "$code" --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "^b$"; then
    test_pass "call_substring_min_core"
  else
    test_fail "call_substring_min_core" "unexpected output: $out"
  fi
}

run_test "call_substring_min_core" test_core_call_substring_min
