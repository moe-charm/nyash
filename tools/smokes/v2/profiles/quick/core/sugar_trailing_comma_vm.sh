#!/bin/bash
# sugar_trailing_comma_vm.sh — Verify trailing commas in literals and calls

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
    return out
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
expected="5"  # 2 (len) + 3 (add) + 10 - 10 = 5
compare_outputs "$expected" "$out" "sugar_trailing_comma_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

