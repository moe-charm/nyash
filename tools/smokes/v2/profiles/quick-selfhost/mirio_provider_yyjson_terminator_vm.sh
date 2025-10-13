#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_body(){
  export HAKO_MIRIO_PROVIDER=yyjson
  bash "$(dirname "$0")/terminator_whitespace_vm.sh"
}

run_test "mirio_provider_yyjson_terminator_vm" test_body || exit 1
print_summary
