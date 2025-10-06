#!/bin/bash
# macro_min_mix_core.sh — core wrapper for minimal json/map/arr macro mix (SKIP gated)

source "$(dirname "$0")/../../lib/test_runner.sh"

require_env || exit 2

if [ "${SMOKES_ENABLE_CORE_MACRO:-0}" != "1" ]; then
  test_skip "core macro mix (set SMOKES_ENABLE_CORE_MACRO=1 to run)"; exit 0
fi

test_core_macro_mix() {
  local code=$(cat << 'NY'
static box Main {
  main() {
    local m = json({ a: 1, b: map({ x: 2 }), c: arr([3,4]) })
    print(m.get("a"))
    return 0
  }
}
NY
)
  out=$(NYASH_MACRO_SELFHOST_MIN=1 NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako NYASH_SYNTAX_SUGAR_LEVEL=full run_nyash_vm -c "$code" --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "Unresolved function\|Parse error"; then
    test_fail "macro_min_mix_core" "resolver error"
  else
    test_pass "macro_min_mix_core"
  fi
}

run_test "macro_min_mix_core" test_core_macro_mix
