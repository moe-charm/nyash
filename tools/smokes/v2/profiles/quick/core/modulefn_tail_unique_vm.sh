#!/bin/bash
# modulefn_tail_unique_vm.sh — ModuleFunction tail-unique resolution (dev-gated)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate: run only when explicitly enabled
if [ "${SMOKES_ENABLE_MODULEFN:-0}" != "1" ]; then
  log_warn "SKIP modulefn_tail_unique_vm (set SMOKES_ENABLE_MODULEFN=1 to run)"
  exit 0
fi

TEST_DIR="/tmp/modulefn_tail_unique_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    // bare function call; unique tail = Beta.util/0
    util()
    return 0
  }
}

static box Beta {
  util() {
    print("ok")
    return 0
  }
}
EOF

# Enable ModuleFunction unification; strict off (uniqueness is guaranteed here)
output=$(NYASH_MIR_CALL_MODULE_FN=1 run_nyash_vm driver.nyash --dev)
output=$(echo "$output" | tail -n 1 | tr -d '\r' | xargs)

if [ "$output" = "ok" ]; then
  log_success "modulefn_tail_unique_vm ok"
  cd /
  rm -rf "$TEST_DIR"
  exit 0
else
  log_error "modulefn_tail_unique_vm expected ok, got: $output"
  cd /
  rm -rf "$TEST_DIR"
  exit 1
fi

