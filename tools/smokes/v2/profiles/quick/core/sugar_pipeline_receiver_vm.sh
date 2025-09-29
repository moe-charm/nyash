#!/bin/bash
# sugar_pipeline_receiver_vm.sh — Verify receiver shorthand x |> .m(a) → x.m(a)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/sugar_pipeline_receiver_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  itos(n) { if n == 0 { return "0" } local v=n local s="" local d="0123456789" loop(v>0){ local k=v%10 s=d.substring(k,k+1)+s v=v/10 } return s }
  // Test obj.m receiver form: x |> obj.m(a) → obj.m(x,a)
  twiceAdd(x, a) { return x + a + a }
  main() {
    local out = 3 |> me.twiceAdd(5) // becomes me.twiceAdd(3,5) => 3 + 5 + 5 = 13
    print(me.itos(out))
    return out
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="13"
compare_outputs "$expected" "$out" "sugar_pipeline_receiver_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
