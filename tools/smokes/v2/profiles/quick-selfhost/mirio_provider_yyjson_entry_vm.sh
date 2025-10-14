#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

# Guard: JSON provider/yyjson path is not stable in quick-selfhost; allow opt-in only
if [ "${SMOKES_ALLOW_JSON_PROVIDER:-0}" != "1" ]; then
  SMOKES_SKIP_CUR_TEST=1; SMOKES_SKIP_REASON="json provider (yyjson) disabled in quick-selfhost";
fi

test_body(){
  export HAKO_MIRIO_PROVIDER=yyjson
  bash "$(dirname "$0")/entry_nonzero_vm.sh"
}

run_test "mirio_provider_yyjson_entry_vm" test_body || exit 1
print_summary
