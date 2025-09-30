#!/bin/bash
# modulefn_tail_ambiguous_vm.sh — Ambiguous tail match should fail in STRICT mode

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate: run only when explicitly enabled
if [ "${SMOKES_ENABLE_MODULEFN:-0}" != "1" ]; then
  log_warn "SKIP modulefn_tail_ambiguous_vm (set SMOKES_ENABLE_MODULEFN=1 to run)"
  exit 0
fi

TEST_DIR="/tmp/modulefn_tail_ambiguous_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    // bare function call; ambiguous tail: Alpha.util/0 and Beta.util/0
    util()
    return 0
  }
}

static box Alpha { util() { print("alpha"); return 0 } }
static box Beta  { util() { print("beta");  return 0 } }
EOF

# STRICT mode expects failure due to ambiguity
set +e
NYASH_MIR_CALL_MODULE_FN=1 NYASH_MIR_CALL_MODULE_FN_STRICT=1 run_nyash_vm driver.nyash --dev >/tmp/_amb_out 2>&1
code=$?
set -e

if [ "$code" -ne 0 ]; then
  log_success "modulefn_tail_ambiguous_vm strict fail-fast ok"
  cd /
  rm -rf "$TEST_DIR"
  exit 0
else
  log_error "modulefn_tail_ambiguous_vm expected failure in STRICT, but succeeded"
  cd /
  rm -rf "$TEST_DIR"
  exit 1
fi

