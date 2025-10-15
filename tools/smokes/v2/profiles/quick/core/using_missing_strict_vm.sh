#!/bin/bash
# using_missing_strict_vm.sh — NYASH_USING_STRICT=1 causes unresolved using to fail-fast

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING_STRICT=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local program=$'using "Foo.Bar" as Baz\nstatic box Main { main(){ return 0 } }'
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -E "^❌ unresolved using 'Foo\\.Bar'|^❌ using:|^❌ Pipeline error: \`using\`|^\`using\` resolution error:" >/dev/null || { echo "$out"; return 1; }
  return 0
}

run_test "using_missing_strict_vm" TEST_main
