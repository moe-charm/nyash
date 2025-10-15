#!/bin/bash
# arity_error_array_push_2args_vm.sh — Array.push with 2 args should error

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/arity_error_array_push_2args_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
static box Main {
  main() {
    local a = new ArrayBox()
    // Wrong arity: push expects 1 arg
    a.push(1, 2)
    print("ok")
    return 0
  }
}
NYEOF

set +e
all=$(run_nyash_vm "$TMP_DIR/driver.nyash" 2>&1)
status=$?
set -e

if [ $status -eq 0 ]; then
  log_error "arity_error_array_push_2args_vm: expected non-zero exit"
  echo "$all" | tail -n 50 >&2
  rm -rf "$TMP_DIR"; exit 1
fi

echo "$all" | grep -Eq "No matching method: ArrayBox\.push\(2 args\)|push expects 1 arg" || {
  log_error "missing arity error message (got:)"
  log_error "missing arity error message"
  echo "$all" | tail -n 80 >&2
  rm -rf "$TMP_DIR"; exit 1
}

log_success "Array.push(2) arity error surfaced"
rm -rf "$TMP_DIR"
exit 0
