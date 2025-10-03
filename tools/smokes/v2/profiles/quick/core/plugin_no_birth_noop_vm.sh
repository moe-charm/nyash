#!/bin/bash
# plugin_no_birth_noop_vm.sh — Birthless plugin should no-op on birth (compat)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Fixture expectation: a dev-only plugin without explicit birth mapping
FIXTURE_SO="plugins/nyash-fixture-plugin/libnyash_fixture_no_birth.so"
if [ ! -f "$FIXTURE_SO" ]; then
  test_skip "plugin_no_birth_noop_vm" "No birthless fixture plugin; skipping (dev-only)" || true
  exit 0
fi

# If the fixture exists, run a minimal script that creates the box and uses a method
script='
using fixture.no_birth as NoBirthBox
static box Main {
  main() {
    local x = new NoBirthBox() // birth should be treated as no-op
    print("ok")
    return 0
  }
}
'

out=$(NYASH_WARN_PLUGIN_NO_BIRTH=0 run_nyash_vm -c "$script" 2>&1 | tail -n 1 | tr -d '\r')
compare_outputs "ok" "$out" "plugin_no_birth_noop_vm" || exit 1

