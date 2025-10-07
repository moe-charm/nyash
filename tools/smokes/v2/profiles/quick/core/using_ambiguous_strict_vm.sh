#!/bin/bash
# using_ambiguous_strict_vm.sh — NYASH_USING_STRICT=1 causes ambiguous using to fail-fast and emit diagnostics JSON

source "$(dirname "$0")/../../../lib/test_runner.sh"
# This smoke is opt-in due to AST prelude interactions in some environments.
if [ "${SMOKES_ENABLE_AMBIG:-0}" != "1" ]; then
  test_skip "using_ambiguous_strict_vm" "Enable with SMOKES_ENABLE_AMBIG=1" || exit 0
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING_STRICT=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Create two temp dirs with the same target file name
  local tmp1=$(mktemp -d)
  local tmp2=$(mktemp -d)
  echo "// box A" > "$tmp1/Foo.hako"
  echo "// box B" > "$tmp2/Foo.hako"
  # Search paths include both temp dirs so resolution finds two candidates
  export NYASH_USING_PATH="$tmp1:$tmp2"
  local program=$'using Foo as F\nstatic box Main { main(){ return 0 } }'
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  # Expect ambiguous error
  echo "$out" | grep -E "^❌ using: ambiguous using 'Foo'|\{\"kind\":\"modules_error\",\"code\":\"ambiguous\"" >/dev/null || { echo "$out"; return 1; }
  return 0
}

run_test "using_ambiguous_strict_vm" TEST_main
