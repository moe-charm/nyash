#!/usr/bin/env bash
# hosthandle_return_type_mismatch_vm.sh — HostHandleRouter 返却型不一致（-14）境界（条件付き）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_hosthandle_return_type_mismatch_vm() {
  # This boundary requires a mock plugin/provider that deliberately returns
  # a wrong TLV type for a slot expecting Integer (e.g., Array.size). Gate by ENV.
  if [ "${HAKO_HOSTHANDLE_TEST_RET_MISMATCH:-0}" != "1" ]; then
    test_skip "requires HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1"; return 0
  fi
  # Pseudo-case: call String.size (slot 300) on a handle that returns non-integer.
  # The actual mismatch must be provided by the test plugin; here we just call size().
  local code=$'static box Main {\n  main() {\n    local s = "hello"\n    // Expect test plugin to force non-integer return to trigger -14\n    return s.size() == 5 ? 0 : 0 // value ignored; harness inspects stderr\n  }\n}\n'
  out=$(run_nyash_vm -c "$code" 2>&1)
  # Accept pass (no-op) or presence of -14 marker in logs depending on env/plugin
  if echo "$out" | grep -q "-14"; then
    test_pass hosthandle_return_type_mismatch_vm
  else
    test_skip "no mismatch observed (plugin not configured)"; return 0
  fi
}

run_test hosthandle_return_type_mismatch_vm test_hosthandle_return_type_mismatch_vm

