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

TMP_DIR="/tmp/selfhost_mir_m2_compare_neg_probe_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using apps/selfhost/vm/boxes/minivm_probe.hako as MiniVmProbe

static box Main {
  main() {
    // Build JSON without escape sequences using a quoted-char variable
    local q = """
    local j = "{" + q + "functions" + q + ":[{" + q + "name" + q + ":" + q + "main" + q + "," + q + "params" + q + ":[] ," + q + "blocks" + q + ":[{" + q + "id" + q + ":0," + q + "instructions" + q + ":["
    j = j + "{" + q + "op" + q + ":" + q + "const" + q + "," + q + "dst" + q + ":1," + q + "value" + q + ":{" + q + "type" + q + ":" + q + "i64" + q + "," + q + "value" + q + ":3}},"
    j = j + "{" + q + "op" + q + ":" + q + "const" + q + "," + q + "dst" + q + ":2," + q + "value" + q + ":{" + q + "type" + q + ":" + q + "i64" + q + "," + q + "value" + q + ":7}},"
    j = j + "{" + q + "op" + q + ":" + q + "binop" + q + "," + q + "op_kind" + q + ":" + q + "Sub" + q + "," + q + "lhs" + q + ":1," + q + "rhs" + q + ":2," + q + "dst" + q + ":3},"
    j = j + "{" + q + "op" + q + ":" + q + "const" + q + "," + q + "dst" + q + ":4," + q + "value" + q + ":{" + q + "type" + q + ":" + q + "i64" + q + "," + q + "value" + q + ":0}},"
    j = j + "{" + q + "op" + q + ":" + q + "compare" + q + "," + q + "cmp" + q + ":" + q + "Lt" + q + "," + q + "lhs" + q + ":3," + q + "rhs" + q + ":4," + q + "dst" + q + ":5},"
    j = j + "{" + q + "op" + q + ":" + q + "ret" + q + "," + q + "value" + q + ":5}] }]}]}"
    local m = MiniVmProbe.probe_compare(j)
    local a = m.get("a")
    local b = m.get("b")
    local r = m.get("r")
    print("A="+MirVmMin._int_to_str(a))
    print("B="+MirVmMin._int_to_str(b))
    print("R="+MirVmMin._int_to_str(r))
    return 0
  }
}

EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 3 | tr -d '
')

# 値抽出
A=$(echo "$out" | grep '^A=' | sed -E 's/^A=//')
B=$(echo "$out" | grep '^B=' | sed -E 's/^B=//')
R=$(echo "$out" | grep '^R=' | sed -E 's/^R=//')

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
