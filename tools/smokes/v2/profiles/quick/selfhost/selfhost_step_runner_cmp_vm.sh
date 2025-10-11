#!/bin/bash
# selfhost_step_runner_cmp_vm.sh — StepRunnerBox で compare→branch の真偽を静的評価

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Gate selfhost StepRunner quick test unless explicitly enabled
if [ "${SMOKES_ENABLE_SELFHOST_STEPRUNNER:-0}" != "1" ]; then
  echo "SKIP: selfhost_step_runner_cmp_vm (set SMOKES_ENABLE_SELFHOST_STEPRUNNER=1 to run)" >&2
  exit 0
fi


export NYASH_USING=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_step_runner_cmp_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "selfhost/compiler/pipeline_v2/emit_compare_box.hako" as E
using "selfhost/vm/boxes/step_runner.hako" as StepRunnerBox

static box Main {
  main() {
    local j = E.emit_compare_cfg3(6, 3, "Gt", 1, 0)
    local b = StepRunnerBox.eval_branch_bool(j)
    print("B=" + (""+b))
    return 0
  }
}
EOF

NYASH_VM_MAX_INSTRUCTIONS=1000000 out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev 2>/dev/null)
b=$(echo "$out" | awk -F= '/^B=/{print $2}' | tr -d '
' | xargs)
if [ "$b" = "1" ]; then
  log_success "StepRunnerBox eval_branch_bool == 1 (Gt 6,3)"
  rm -rf "$TMP_DIR"; exit 0
else
  log_error "StepRunnerBox eval_branch_bool expected 1, got: ${b:-<empty>}"
  echo "$out" >&2
  rm -rf "$TMP_DIR"; exit 1
fi
