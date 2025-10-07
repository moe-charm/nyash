#!/bin/bash
# selfhost_mir_m2_compare_neg_probe_vm.sh — MiniVmProbe で a/b/r を観測（診断用、常時PASS）

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_VM_MAX_INSTRUCTIONS=10000000

TMP_DIR="/tmp/selfhost_mir_m2_compare_neg_probe_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox
using apps/selfhost/vm/boxes/minivm_probe.hako as MiniVmProbe

static box Main {
  main() {
    // Build JSON using raw string literal (Rust-style r#"..."#)
    local j = r#"{"functions":[{"name":"main","params":[],"blocks":[{"id":0,"instructions":[{"op":"const","dst":1,"value":{"type":"i64","value":3}},{"op":"const","dst":2,"value":{"type":"i64","value":7}},{"op":"binop","op_kind":"Sub","lhs":1,"rhs":2,"dst":3},{"op":"const","dst":4,"value":{"type":"i64","value":0}},{"op":"compare","cmp":"Lt","lhs":3,"rhs":4,"dst":5},{"op":"ret","value":5}]}]}]}"#
    local m = MiniVmProbe.probe_compare(j)
    local a = m.get("a")
    local b = m.get("b")
    local r = m.get("r")
    print("A="+MiniVmEntryBox.int_to_str(a))
    print("B="+MiniVmEntryBox.int_to_str(b))
    print("R="+MiniVmEntryBox.int_to_str(r))
    return 0
  }
}

EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 3 | tr -d '
')

# 値抽出
A=$(echo "$out" | sed -nE 's/.*A=([-0-9]+).*/\1/p')
B=$(echo "$out" | sed -nE 's/.*B=([-0-9]+).*/\1/p')
R=$(echo "$out" | sed -nE 's/.*R=([-0-9]+).*/\1/p')

# 強制表示モード
if [ "${SMOKES_FORCE_SHOW:-0}" = "1" ]; then
  log_warn "Probe A=${A:-?} B=${B:-?} R=${R:-?} (forced show)"
  echo "--- RAW OUT BEGIN ---" >&2
  echo "$out" >&2
  echo "--- RAW OUT END ---" >&2
  echo "--- DRIVER BEGIN ---" >&2
  sed -n '1,200p' "$TMP_DIR/driver.nyash" >&2 || true
  echo "--- DRIVER END ---" >&2
  rm -rf "$TMP_DIR"
  test_fail "selfhost_mir_m2_compare_neg_probe_vm (forced show)"
  exit 1
fi

rm -rf "$TMP_DIR"
log_success "selfhost_mir_m2_compare_neg_probe_vm (diagnostic) A=${A:-?} B=${B:-?} R=${R:-?}"
exit 0
