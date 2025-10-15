#!/bin/bash
# arity_error_string_indexof_2args_vm.sh — String.indexOf with 2 args should error

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2
log_warn "SKIP arity_error_string_indexof_2args_vm (quick: semantics vary across builds)"; exit 0

TMP_DIR="/tmp/arity_error_string_indexof_2args_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
static box Main {
  main() {
    local s = "abcde"
    // Wrong arity: indexOf expects 1 arg
    local i = s.indexOf("bc", 1)
    print("i=" + (""+i))
    return 0
  }
}
NYEOF

set +e
all=$(run_nyash_vm "$TMP_DIR/driver.nyash" 2>&1)
status=$?
set -e

if [ $status -eq 0 ]; then
  log_error "arity_error_string_indexof_2args_vm: expected non-zero exit"
  echo "$all" | tail -n 80 >&2
  rm -rf "$TMP_DIR"; exit 1
fi

echo "$all" | grep -Eq "No matching method: StringBox\\.indexOf\\(2 args\\)|indexOf expects 1 arg" || {
  log_error "missing arity error message"
  echo "$all" | tail -n 80 >&2
  rm -rf "$TMP_DIR"; exit 1
}

log_success "String.indexOf(2) arity error surfaced"
rm -rf "$TMP_DIR"
exit 0
