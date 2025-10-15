#!/bin/bash
# selfhost_mir_m2_scanner_step_vm.sh — Placeholder smoke for InstructionScannerBox (Box‑First extraction)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_MODULES="${NYASH_MODULES:+$NYASH_MODULES,}selfhost.vm.scanner=apps/selfhost/vm/boxes/instruction_scanner.hako"
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
if [ "${SMOKES_SELFHOST_M2M3_ENABLE:-0}" != "1" ]; then test_skip "selfhost M2/M3 gated (set SMOKES_SELFHOST_M2M3_ENABLE=1)"; exit 0; fi

# Placeholder: enable after InstructionScannerBox is stabilized under dev operator profile
test_skip "selfhost_mir_m2_scanner_step_vm (pending stabilization)" \
  "Enable after operator prelude/adopt are consistent in dev profile"
exit 0
