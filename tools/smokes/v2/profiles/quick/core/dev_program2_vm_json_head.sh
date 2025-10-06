#!/bin/bash
# dev_program2_vm_json_head.sh — Dev-only: Program2 VM JSON head non-empty check (lightweight)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_program2_vm_json_head() {
  # Default: SKIP unless explicitly enabled
  if [ "${SMOKES_ENABLE_DEV_PROGRAM2:-0}" != "1" ]; then
    test_skip "dev_program2_vm_json_head (set SMOKES_ENABLE_DEV_PROGRAM2=1 to enable)"
    return 0
  fi

  # Enable using for dev driver
  local out
  out=$(NYASH_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
        run_nyash_vm "apps/dev/debug_program2_vm.nyash" 2>&1 || true)

  # Expect five JSON head lines
  local count
  count=$(echo "$out" | grep -E "^p2 (vm|if|loop|nested-if|concat) head = \{\"version\":0,\"kind\":\"Program\",\"b" | wc -l | tr -d ' ')
  if [ "$count" -ge 5 ]; then
    return 0
  fi
  echo "$out" >&2
  echo "[FAIL] dev_program2_vm_json_head: expected 5 head lines, got $count" >&2
  return 1
}

run_test "dev_program2_vm_json_head" test_program2_vm_json_head

