#!/bin/bash
# json_v1_mir_call_vm.sh — Verify JSON v1 emits unified mir_call entries (dev-gated)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=1  # Go through PyVM bridge to generate MIR JSON via bin emitter
require_env || exit 2
preflight_plugins || exit 2

TEST_DIR="/tmp/json_v1_mir_call_vm_$$"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

cat > driver.nyash << 'EOF'
static box Main {
  main() {
    print("hello")
    return 0
  }
}
EOF

# Run with JSON v1 schema enabled
set +e
NYASH_DISABLE_PLUGINS=1 NYASH_JSON_SCHEMA_V1=1 output=$(run_nyash_vm driver.nyash --dev 2>&1)
code=$?
set -e

# Check that the PyVM bridge JSON exists and contains mir_call
JSON_PATH="tmp/nyash_pyvm_mir.json"
if [ ! -f "$JSON_PATH" ]; then
  echo "$output" | tail -n 50 >&2
  log_error "json_v1_mir_call_vm: expected $JSON_PATH (PyVM bridge JSON)"
  cd /
  rm -rf "$TEST_DIR"
  exit 1
fi

if grep -q '"op"\s*:\s*"mir_call"' "$JSON_PATH"; then
  log_success "json_v1_mir_call_vm ok"
  cd /
  rm -rf "$TEST_DIR"
  exit 0
else
  tail -n 80 "$JSON_PATH" >&2
  log_error "json_v1_mir_call_vm: expected unified mir_call in JSON v1"
  cd /
  rm -rf "$TEST_DIR"
  exit 1
fi

