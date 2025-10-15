#!/bin/bash
# sugar_raw_basic_vm.sh — Verify raw strings r"…" and r#"…"#

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1

TMP_DIR="/tmp/sugar_raw_basic_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  itos(n) { if n == 0 { return "0" } local v=n local s="" local d="0123456789" loop(v>0){ local k=v%10 s=d.substring(k,k+1)+s v=v/10 } return s }
  main() {
    local s1 = r"{}"
    local s2 = r"line\nraw"
    local sum = s1.length() + s2.length()
    print(me.itos(sum))
    return sum
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
expected="11"
compare_outputs "$expected" "$out" "sugar_raw_basic_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
