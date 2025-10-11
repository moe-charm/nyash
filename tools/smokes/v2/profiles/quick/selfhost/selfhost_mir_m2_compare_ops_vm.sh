#!/bin/bash
# selfhost_mir_m2_compare_ops_vm.sh — Minimal compare ops (Eq/Ne/Lt/Le/Gt/Ge) → 0/1
# tags: selfhost

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

TMP_DIR="/tmp/selfhost_mir_m2_compare_ops_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox
using "selfhost/shared/json/mir_builder_min.hako" as MirJsonBuilderMin

static box Main {
  main() {
    // ops: Eq Ne Lt Le Gt Ge on a=5, b=4
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
      local builder = new MirJsonBuilderMin()
      builder.start_module()
      builder.start_function("main")
      builder.start_block(0)
      builder.add_const(1, 5)
      builder.add_const(2, 4)
      builder.add_compare(op, 1, 2, 3)
      builder.add_ret(3)
      builder.end_all()
      local j = builder.to_string()
      local v = MirVmMin._run_min(j)
      if i > 0 { out = out + " " }
      out = out + MiniVmEntryBox.int_to_str(v)
      i = i + 1
    }
    print(out)
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected_line="0 1 0 0 1 1"
compare_outputs "$expected_line" "$out" "selfhost_mir_m2_compare_ops_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
