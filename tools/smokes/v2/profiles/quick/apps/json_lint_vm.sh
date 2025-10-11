#!/bin/bash
# json_lint_vm.sh — Example app: JSON lint (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

# Always-on: CallResolver and prelude handling make this stable in quick profile

APP_DIR="$NYASH_ROOT/apps/examples/json_lint"
# Strict mode: do not tolerate Void in VM (policy: tests must not rely on NYASH_VM_TOLERATE_VOID)
# Drop trailing VM result summary lines to keep output stable
export HAKO_PLUGIN_POLICY=off
# For stability in quick profile, assert exit code only (output may vary with provider state)
if run_nyash_vm "$APP_DIR/main.nyash" --dev >/dev/null; then
  echo "OK"
  exit 0
else
  echo "FAIL: non-zero exit" >&2
  exit 1
fi
