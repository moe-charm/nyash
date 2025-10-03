#!/bin/bash
# selfhost_mir_m2_compare_eq_boundary_vm.sh — a=b equality boundary (Eq/Ne/Lt/Le/Gt/Ge)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_compare_eq_boundary_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // ops: Eq Ne Lt Le Gt Ge on a=b=4
    local ops = new ArrayBox()
    ops.push("Eq")
    ops.push("Ne")
    ops.push("Lt")
    ops.push("Le")
    ops.push("Gt")
    ops.push("Ge")
    local i = 0
    local out = ""
    loop (i < ops.length()) {
      local op = ops.get(i)
      // Build JSON once per op
      local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":4}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":4}},{\"op\":\"compare\",\"cmp\":\"" + op + "\",\"lhs\":1,\"rhs\":2,\"dst\":3},{\"op\":\"ret\",\"value\":3}]}]}]}"
      local v = MirVmMin._run_min(j)
      if i > 0 { out = out + " " }
      out = out + MirVmMin._int_to_str(v)
      i = i + 1
    }
    print(out)
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected_line="1 0 0 1 0 1"
compare_outputs "$expected_line" "$out" "selfhost_mir_m2_compare_eq_boundary_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
