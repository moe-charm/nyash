#!/bin/bash
# modulefn_llvm_trace.sh — Verify LLVM call trace prints ModuleFunction

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate
if [ "${SMOKES_ENABLE_MODULEFN:-0}" != "1" ]; then
  log_warn "SKIP modulefn_llvm_trace (set SMOKES_ENABLE_MODULEFN=1 to run)"
  exit 0
fi

TEST_DIR="/tmp/modulefn_llvm_trace_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    util()
    return 0
  }
}

static box Util {
  util() { return 0 }
}
EOF

set +e
NYASH_CALL_TRACE=1 output=$(run_nyash_llvm driver.nyash --dev 2>&1)
code=$?
set -e

echo "$output" | grep -q '"callee":"ModuleFunction:Util.util/0"'
if [ $? -eq 0 ]; then
  log_success "modulefn_llvm_trace ok"
  cd /
  rm -rf "$TEST_DIR"
  exit 0
else
  echo "$output" | tail -n 50 >&2
  log_error "modulefn_llvm_trace expected ModuleFunction trace"
  cd /
  rm -rf "$TEST_DIR"
  exit 1
fi

