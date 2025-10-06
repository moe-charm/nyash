#!/bin/bash
# sugar_pipeline_placeholder_vm.sh — Verify placeholder '_' in pipeline RHS

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/sugar_pipeline_placeholder_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  itos(n) { if n == 0 { return "0" } local v=n local s="" local d="0123456789" loop(v>0){ local k=v%10 s=d.substring(k,k+1)+s v=v/10 } return s }
  plus(a, b) { return a + b }
  mix(a, b, c) { return a * 100 + b * 10 + c }
  main() {
    local r1 = 3 |> me.plus(_, 4)     // 3 + 4 = 7
    local r2 = 5 |> me.mix(1, _, 2)   // 5 placed in middle → 1,5,2 => 152 (by mix's encoding)
    print(me.itos(r1 + r2))           // 7 + 152 = 159
    return r1 + r2
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
expected="159"
compare_outputs "$expected" "$out" "sugar_pipeline_placeholder_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

