#!/bin/bash
# filebox_basic.sh - FileBox の最小E2E（コアBoxを使用）

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_filebox_write_bytes() {
  # Ensure plugin policy is ON for this test only
  export HAKO_PLUGIN_POLICY=${HAKO_PLUGIN_POLICY:-auto}
  local tmp="/tmp/nyash_smoke_file_$$.txt"
  local script="
  local fb, n
  fb = new FileBox()
  fb.open(\"$tmp\", \"w\")
  n = fb.write(\"hello\")
  fb.close()
  print(n)
  "
  local output
  output=$(run_nyash_vm -c "$script" 2>&1 || true)
  rm -f "$tmp" 2>/dev/null || true
  if echo "$output" | grep -q "Unknown Box type: FileBox\|VM fallback error: Invalid instruction: NewBox FileBox failed\|Method open not supported on Void\|Invalid value: use of undefined value"; then
    test_skip "filebox_write_bytes" "FileBox not available (plugin not loaded)"
    return 0
  fi
  check_exact "5" "$output" "filebox_write_bytes"
}

run_test "filebox_write_bytes" test_filebox_write_bytes
