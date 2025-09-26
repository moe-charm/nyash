#!/usr/bin/env bash
# Nyash dev environment convenience script
# Usage: source tools/dev_env.sh [profile]
# Profiles:
#   pyvm     - Favor PyVM for VM and Bridge
#   bridge   - Bridge-only helpers (keep interpreter)
#   phi_off  - PHI-less MIR (edge-copy) + verifier relax; harness on
#   reset    - Unset variables set by this script

set -euo pipefail

activate_pyvm() {
  export NYASH_DISABLE_PLUGINS=1
  export NYASH_VM_USE_PY=1
  export NYASH_PIPE_USE_PYVM=1
  export NYASH_NY_COMPILER_TIMEOUT_MS=${NYASH_NY_COMPILER_TIMEOUT_MS:-2000}
  echo "[dev-env] PyVM profile activated" >&2
}

activate_bridge() {
  export NYASH_DISABLE_PLUGINS=1
  unset NYASH_VM_USE_PY || true
  export NYASH_NY_COMPILER_TIMEOUT_MS=${NYASH_NY_COMPILER_TIMEOUT_MS:-2000}
  echo "[dev-env] Bridge profile activated (interpreter for pipe)" >&2
}

reset_env() {
  unset NYASH_VM_USE_PY || true
  unset NYASH_PIPE_USE_PYVM || true
  unset NYASH_DISABLE_PLUGINS || true
  unset NYASH_NY_COMPILER_TIMEOUT_MS || true
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

case "${1:-pyvm}" in
  pyvm) activate_pyvm ;;
  bridge) activate_bridge ;;
  phi_off) activate_phi_off ;;
  reset) reset_env ;;
  *) echo "usage: source tools/dev_env.sh [pyvm|bridge|phi_off|reset]" >&2 ;;
esac
