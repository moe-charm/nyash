#!/bin/bash
# dep_tree_summary_core.sh — Verify dep_tree summary (--summary) produces sane JSON (SKIP gated)

source "$(dirname "$0")/../../lib/test_runner.sh"

require_env || exit 2

if [ "${SMOKES_ENABLE_DEP_SUMMARY:-0}" != "1" ]; then
  test_skip "dep_tree summary (--summary) (set SMOKES_ENABLE_DEP_SUMMARY=1 to run)"; exit 0
fi

test_dep_tree_summary_core() {
  local entry="apps/selfhost/ny-parser-nyash/main.nyash"
  local out
  out=$(run_nyash_vm apps/selfhost/tools/dep_tree_main.hako "$entry" --summary --dev 2>&1 | filter_noise | tail -n 1)
  if echo "$out" | jq -e '.nodes >= 1 and (.include_resolved|.>=0) and (.include_missing|.>=0) and (.using_resolved|.>=0) and (.using_unresolved|.>=0)' >/dev/null 2>&1; then
    test_pass "dep_tree_summary_core"
  else
    test_fail "dep_tree_summary_core" "summary JSON missing keys or invalid: $out"
  fi
}

run_test "dep_tree_summary_core" test_dep_tree_summary_core
