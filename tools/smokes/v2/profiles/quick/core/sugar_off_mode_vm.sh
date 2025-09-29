#!/bin/bash
# sugar_off_mode_vm.sh — Sugar disabled mode should reject sugar constructs

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/sugar_off_mode_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  itos(n) { if n == 0 { return "0" } local v=n local s="" local d="0123456789" loop(v>0){ local k=v%10 s=d.substring(k,k+1)+s v=v/10 } return s }
  f(x) { return x }
  main() {
    // pipeline sugar should be rejected when sugar level=off
    local r = 1 |> me.f()
    print(me.itos(r))
    return r
  }
}
EOF

# Disable sugar via ENV
export NYASH_SYNTAX_SUGAR_LEVEL=off
set +e
run_nyash_vm "$TMP_DIR/driver.nyash" --dev >/dev/null 2>&1
rc=$?
set -e

if [ $rc -eq 0 ]; then
  log_error "sugar_off_mode_vm: expected parser error but got success"
  cd /; rm -rf "$TMP_DIR"; exit 1
else
  log_success "sugar_off_mode_vm (error as expected)"
fi

rm -rf "$TMP_DIR"
exit 0

