#!/bin/bash
# map_pair_spec_core.sh — Verify pair returns are map({arr,len}) (SKIP gated)

source "$(dirname "$0")/../../lib/test_runner.sh"

require_env || exit 2

if [ "${SMOKES_ENABLE_CORE_MAP_PAIR:-0}" != "1" ]; then
  test_skip "core map({arr,len}) spec (set SMOKES_ENABLE_CORE_MAP_PAIR=1 to run)"; exit 0
fi

test_core_map_pair() {
  local code=$(cat << 'NY'
using selfhost.tools.dep_tree_simple as Dep

static box Main {
  main() {
    local p = Dep.split_lines("a\nb")
    local arr = p.get("arr")
    local n = p.get("len")
    if arr != null && arr.size != null && arr.size() == n { print("OK") }
    return 0
  }
}
NY
)
  out=$(NYASH_MACRO_SELFHOST_MIN=1 NYASH_MACRO_BOX_CHILD_RUNNER=0 NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako NYASH_SYNTAX_SUGAR_LEVEL=full run_nyash_vm -c "$code" --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "OK"; then
    test_pass "map_pair_spec_core"
  else
    test_fail "map_pair_spec_core" "pair return not map-like or size mismatch"
  fi
}

run_test "map_pair_spec_core" test_core_map_pair
