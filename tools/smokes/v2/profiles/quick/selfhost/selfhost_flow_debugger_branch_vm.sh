#!/bin/bash
# selfhost_flow_debugger_branch_vm.sh — FlowDebugBox で CFG 妥当性と op 列挙を検査

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_ENABLE_USING=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_flow_debugger_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/pipeline_v2/emit_compare_box.hako" as E
using "apps/selfhost/vm/boxes/flow_debugger.hako" as FlowDebugBox

static box Main {
  main() {
    // 6 > 3, materialize=1 → copy + branch + ret
    local j = E.emit_compare_cfg3(6, 3, "Gt", 1, 0)
    // 構造検査
    FlowDebugBox.summarize_ops(j, 10)
    local errs = FlowDebugBox.validate_cf_targets(j)
    // errs==0 を期待
    print("ERRS=" + (""+errs))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev 2>/dev/null)
errs=$(echo "$out" | awk -F= '/^ERRS=/{print $2}' | tr -d '' | xargs)
if [ "$errs" = "0" ]; then
  log_success "FlowDebugBox validated branch/jump targets (errs=0)"
  rm -rf "$TMP_DIR"; exit 0
else
  log_error "FlowDebugBox reported errors: $errs"
  echo "$out" >&2
  rm -rf "$TMP_DIR"; exit 1
fi
