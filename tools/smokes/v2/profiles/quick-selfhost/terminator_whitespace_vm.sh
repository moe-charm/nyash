#!/bin/bash
source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

function test_body(){
  ensure_hako_toml
  local tmp
  tmp=$(mktemp)
  cat > "$tmp" << 'SRC'
using "selfhost/hakorune-vm/hakorune_vm_core.hako" as HakoruneVmCore
static box Main {
  main() {
    local j = "{\"name\":\"main\",\"entry\":0,\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"ret\",\"value\":null}],\"terminator\":{\"op\":\"ret\",\"value\":null}}]}"
    return HakoruneVmCore.run(j)
  }
}
SRC
  # Expect no stdout output on success
  out=$(HAKO_ALLOW_USING_FILE=1 "$NYASH_BIN" --backend nyvm "$tmp" 2>&1 | filter_noise | grep -v '^Unknown backend:')
  compare_outputs "" "${out}" "terminator_whitespace_vm"
}

run_test "terminator_whitespace_vm" test_body || exit 1
print_summary
