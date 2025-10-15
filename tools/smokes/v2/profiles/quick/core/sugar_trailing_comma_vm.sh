#!/bin/bash
# sugar_trailing_comma_vm.sh — Verify trailing commas in literals and calls (rc-only in quick)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1

TMP_DIR="/tmp/sugar_trailing_comma_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  itos(n) { if n == 0 { return "0" } local v=n local s="" local d="0123456789" loop(v>0){ local k=v%10 s=d.substring(k,k+1)+s v=v/10 } return s }
  add(a, b) { return a + b }
  main() {
    local arr = [1, 2,]
    local m = {"a": 10,}
    local r = me.add(1, 2,)
    local out = arr.length() + r + m.get("a") - 10
    print(me.itos(out))
    return 0
  }
}
EOF

if run_nyash_vm "$TMP_DIR/driver.nyash" --dev >/dev/null; then
  rm -rf "$TMP_DIR"; exit 0
else
  rc=$?
  rm -rf "$TMP_DIR"; echo "FAIL: rc=$rc" >&2; exit 1
fi
