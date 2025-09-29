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
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ensure_app_bin_dir() {
  # Default artifacts location for app_* outputs (can be overridden by caller)
  export APP_BIN_DIR=${APP_BIN_DIR:-artifacts/apps}
  mkdir -p "${APP_BIN_DIR}"
  # Enable tight navigation mode by default (can disable with DEV_TIGHT=0)
  if [[ "${DEV_TIGHT:-1}" == "1" ]]; then
    if [[ -f "${SCRIPT_DIR}/dev/tight_mode.sh" ]]; then
      # shellcheck source=/dev/null
      source "${SCRIPT_DIR}/dev/tight_mode.sh"
    fi
  fi
}

activate_pyvm() {
  ensure_app_bin_dir
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
  ensure_app_bin_dir
  unset NYASH_DISABLE_PLUGINS || true
  unset NYASH_VM_USE_PY || true
  export NYASH_NY_COMPILER_TIMEOUT_MS=${NYASH_NY_COMPILER_TIMEOUT_MS:-2000}
  export NYASH_MIR_UNIFIED_CALL=${NYASH_MIR_UNIFIED_CALL:-1}
  export NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1
  echo "[dev-env] Bridge profile activated (interpreter for pipe; plugins ON)" >&2
}

reset_env() {
  unset APP_BIN_DIR || true
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
  ensure_app_bin_dir
  export NYASH_MIR_NO_PHI=1
  export NYASH_VERIFY_ALLOW_NO_PHI=1
  export NYASH_LLVM_USE_HARNESS=1
  echo "[dev-env] PHI-off (edge-copy) profile activated (harness on)" >&2
}

activate_opbox() {
  ensure_app_bin_dir
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
  selfhost)
    ensure_app_bin_dir
    # Focus on self-hosting workflows; keep toggles minimal and reversible
    export NYASH_COMPILER_TRACK=1
    export DEV_TIGHT=${DEV_TIGHT:-1}
    echo "[dev-env] Selfhost profile activated (APP_BIN_DIR=${APP_BIN_DIR}, COMPILER_TRACK=1, tight=${DEV_TIGHT})" >&2
    echo "[hint] Run selfhost smokes: APP_BIN_DIR=${APP_BIN_DIR} tools/selfhost_smokes.sh quick|integration" >&2
    ;;
  pyvm) activate_pyvm ;;
  bridge) activate_bridge ;;
  phi_off) activate_phi_off ;;
  opbox) activate_opbox ;;
  reset) reset_env ;;
  *) echo "usage: source tools/dev_env.sh [pyvm|bridge|phi_off|opbox|reset]" >&2 ;;
esac
