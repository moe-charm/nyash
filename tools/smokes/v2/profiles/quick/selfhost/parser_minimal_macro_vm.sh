#!/bin/bash
# parser_minimal_macro_vm.sh — selfhost minimal parser uses map! sugar internally

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2

# Gate: this smoke touches using-prelude on a Stage-0 file that may not
# accept ";" in the minimal prelude scanner. Enable explicitly when needed.
if [ "${SMOKES_ENABLE_SELFHOST_MIN_PARSER:-0}" != "1" ]; then
  test_skip "selfhost minimal parser macro (set SMOKES_ENABLE_SELFHOST_MIN_PARSER=1 to run)"; exit 0
fi

test_parser_minimal_macro() {
  code=$(cat << 'NY'
using "./apps/selfhost/ny-parser-nyash/parser_minimal.nyash" as ParserV0

static box Main {
  main() {
    local out = ParserV0.parse_program("return 1+2")
    print(out.get("kind"))
    return 0
  }
}
NY
)
  out=$(run_nyash_vm -c "$code" --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "Program"; then
    test_pass "parser_minimal_macro_vm"
  else
    test_fail "parser_minimal_macro_vm" "no Program in output"
  fi
}

run_test "parser_minimal_macro_vm" test_parser_minimal_macro
