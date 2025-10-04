#!/bin/bash
# selfhost_localssa_phi_head_copy_vm.sh — branch(cond from prev block) with PHI head → copy inserted after PHI
# tags: selfhost,trace

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

# Experimental guard: LocalSSA copy-after-PHI behavior may be pending
if [[ "${SMOKES_ENABLE_LOCALSSA_PHI:-0}" != "1" ]]; then
  test_skip "selfhost_localssa_phi_head_copy_vm" "enable with SMOKES_ENABLE_LOCALSSA_PHI=1"
  exit 0
fi

TMP_DIR="/tmp/selfhost_localssa_phi_head_copy_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/builder/ssa/local.hako" as LocalSSA

static box Main {
  main() {
    // Block 0: const cond=1; jump 1
    // Block 1: phi dst=10 (dummy), then branch(cond=1) — ensure copy after PHI and before branch
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":1}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"phi\",\"dst\":10,\"incoming\":[{\"block\":0,\"value\":1}]},{\"op\":\"branch\",\"cond\":1,\"then\":2,\"else\":3}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},"
    j = j + "{\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    local out = LocalSSA.ensure_cond(j)
    print(out)
    return 0
  }
}
EOF

json=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r')

# Expect: phi ... copy ... branch order
if ! echo "$json" | grep -q '"op":"phi".*"op":"copy".*"op":"branch"'; then
  log_error "expected copy after PHI and before branch"
  echo "$json" | tail -n 1 >&2
  rm -rf "$TMP_DIR"; exit 1
fi

log_success "selfhost_localssa_phi_head_copy_vm"
rm -rf "$TMP_DIR"
exit 0
