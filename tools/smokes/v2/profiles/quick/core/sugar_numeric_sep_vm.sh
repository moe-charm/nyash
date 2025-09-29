#!/bin/bash
# sugar_numeric_sep_vm.sh — Verify numeric separators in integer/float

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1

TMP_DIR="/tmp/sugar_numeric_sep_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  itos(n) { if n == 0 { return "0" } local v=n local s="" local d="0123456789" loop(v>0){ local k=v%10 s=d.substring(k,k+1)+s v=v/10 } return s }
  main() {
    local a = 1_000_000
    local b = 2
    local c = 3.141_592
    // ignore c; just ensure float tokenizes
    local r = a + b
    print(me.itos(r))
    return r
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1000002"
compare_outputs "$expected" "$out" "sugar_numeric_sep_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

