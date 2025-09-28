#!/usr/bin/env bash
# Nyash dev environment convenience script
# Usage: source tools/dev_env.sh [profile]
# Profiles:
#   pyvm     - Favor PyVM for VM and Bridge
#   bridge   - Bridge-only helpers (keep interpreter)
#   phi_off  - PHI-less MIR (edge-copy) + verifier relax; harness on
#   opbox    - Enable Operator Boxes (Stringify/Compare/Add) with adopt; AST using ON
#   reset    - Unset variables set by this script

set -euo pipefail

activate_pyvm() {
  unset NYASH_DISABLE_PLUGINS || true
  export NYASH_VM_USE_PY=1
  export NYASH_PIPE_USE_PYVM=1
  export NYASH_NY_COMPILER_TIMEOUT_MS=${NYASH_NY_COMPILER_TIMEOUT_MS:-2000}
  # Unified Call: default ON (explicit), and suppress legacy rewrite to avoid duplication
  export NYASH_MIR_UNIFIED_CALL=${NYASH_MIR_UNIFIED_CALL:-1}
  export NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1
  echo "[dev-env] PyVM profile activated (plugins ON)" >&2
}

activate_bridge() {
  unset NYASH_DISABLE_PLUGINS || true
  unset NYASH_VM_USE_PY || true
  export NYASH_NY_COMPILER_TIMEOUT_MS=${NYASH_NY_COMPILER_TIMEOUT_MS:-2000}
  export NYASH_MIR_UNIFIED_CALL=${NYASH_MIR_UNIFIED_CALL:-1}
  export NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1
  echo "[dev-env] Bridge profile activated (interpreter for pipe; plugins ON)" >&2
}

reset_env() {
  unset NYASH_VM_USE_PY || true
  unset NYASH_PIPE_USE_PYVM || true
  unset NYASH_DISABLE_PLUGINS || true
  unset NYASH_NY_COMPILER_TIMEOUT_MS || true
  unset NYASH_MIR_UNIFIED_CALL || true
  unset NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE || true
  unset NYASH_MIR_NO_PHI || true
  unset NYASH_VERIFY_ALLOW_NO_PHI || true
  unset NYASH_LLVM_USE_HARNESS || true
  echo "[dev-env] environment reset" >&2
}

activate_phi_off() {
  export NYASH_MIR_NO_PHI=1
  export NYASH_VERIFY_ALLOW_NO_PHI=1
  export NYASH_LLVM_USE_HARNESS=1
  echo "[dev-env] PHI-off (edge-copy) profile activated (harness on)" >&2
}

activate_opbox() {
  export NYASH_USING_AST=1
  # Runtime operator boxes
  export NYASH_OPERATOR_BOX_STRINGIFY=1
  export NYASH_OPERATOR_BOX_COMPARE=1
  export NYASH_OPERATOR_BOX_ADD=1
  export NYASH_OPERATOR_BOX_ALL=1
  export NYASH_OPERATOR_BOX_COMPARE_ADOPT=1
  export NYASH_OPERATOR_BOX_ADD_ADOPT=1
  # Builder lowering to operator calls
  export NYASH_BUILDER_OPERATOR_BOX_COMPARE_CALL=1
  export NYASH_BUILDER_OPERATOR_BOX_ADD_CALL=1
  export NYASH_BUILDER_OPERATOR_BOX_ALL_CALL=1
  # Unified call and legacy suppression
  export NYASH_MIR_UNIFIED_CALL=${NYASH_MIR_UNIFIED_CALL:-1}
  export NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1
  echo "[dev-env] Operator Boxes (stringify/compare/add) enabled (adopt+builder-call)" >&2
}

case "${1:-pyvm}" in
  pyvm) activate_pyvm ;;
  bridge) activate_bridge ;;
  phi_off) activate_phi_off ;;
  opbox) activate_opbox ;;
  reset) reset_env ;;
  *) echo "usage: source tools/dev_env.sh [pyvm|bridge|phi_off|opbox|reset]" >&2 ;;
esac
