#!/bin/bash
# selfhost_localssa_same_block_no_copy_vm.sh — cond defined in same block → no materialize copy
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_localssa_same_block_no_copy_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/builder/ssa/local.hako" as LocalSSA

static box Main {
  main() {
    // Block 0: const→const→compare→branch (cond is defined in the same block)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j = j + "{\\\"op\\\":\\\"const\\\",\\\"dst\\\":1,\\\"value\\\":{\\\"type\\\":\\\"i64\\\",\\\"value\\\":5}},"
    j = j + "{\\\"op\\\":\\\"const\\\",\\\"dst\\\":2,\\\"value\\\":{\\\"type\\\":\\\"i64\\\",\\\"value\\\":4}},"
    j = j + "{\\\"op\\\":\\\"compare\\\",\\\"cmp\\\":\\\"Gt\\\",\\\"lhs\\\":1,\\\"rhs\\\":2,\\\"dst\\\":3},"
    j = j + "{\\\"op\\\":\\\"branch\\\",\\\"cond\\\":3,\\\"then\\\":1,\\\"else\\\":2}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"const\",\"dst\":6,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":3}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"const\",\"dst\":6,\"value\":{\"type\":\"i64\",\"value\":0}},{\"op\":\"jump\",\"target\":3}]},"
    j = j + "{\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":6}]}]}]}"
    local out = LocalSSA.ensure_cond(j)
    print(out)
    return 0
  }
}
EOF

json=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' )

# Expect: no copy op since cond is defined in the same block
if echo "$json" | grep -q '"op":"copy"'; then
  log_error "unexpected materialize copy in same-block cond case"
  echo "$json" | tail -n 1 >&2
  rm -rf "$TMP_DIR"; exit 1
fi

log_success "selfhost_localssa_same_block_no_copy_vm"
rm -rf "$TMP_DIR"
exit 0

