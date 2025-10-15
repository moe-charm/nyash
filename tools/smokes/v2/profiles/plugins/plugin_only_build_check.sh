#!/usr/bin/env bash
# plugin_only_build_check.sh — Build-only check for plugin-only line (legacy-boxes OFF)
# SMOKES_TIMEOUT=180
# SMOKES_ENV+=SMOKES_TIMEOUT_SEC=180

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_plugin_only_build_check() {
  # Build-only; do not run artifacts. Keep as an optional developer check.
  # If environment is not suitable (no cargo), require_env already fails.
  if [ "${SMOKES_SKIP_BUILD_ONLY:-0}" = "1" ]; then
    test_skip "build-only checks disabled via SMOKES_SKIP_BUILD_ONLY=1"; return 0
  fi
  # Try a fast build (reuse cache). Allow long timeout when set by caller.
  local cmd=(cargo build --release --no-default-features -F cli,plugins,host-anchors)
  if [ -n "${SMOKES_TIMEOUT_SEC:-}" ] && [ "${SMOKES_TIMEOUT_SEC}" != "0" ]; then
    timeout -s KILL "${SMOKES_TIMEOUT_SEC}s" "${cmd[@]}" >/dev/null 2>&1 || {
      test_fail "plugin-only build failed or timed out"; return 1; }
  else
    "${cmd[@]}" >/dev/null 2>&1 || { test_fail "plugin-only build failed"; return 1; }
  fi
  test_pass plugin_only_build_check
}

run_test plugin_only_build_check test_plugin_only_build_check
