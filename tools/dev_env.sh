#!/usr/bin/env bash
# Nyash dev environment convenience script
# Usage: source tools/dev_env.sh [profile]
# Profiles:
#   pyvm   - Favor PyVM for VM and Bridge
#   bridge - Bridge-only helpers (keep interpreter)
#   reset  - Unset variables set by this script

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
  echo "[dev-env] environment reset" >&2
}

case "${1:-pyvm}" in
  pyvm) activate_pyvm ;;
  bridge) activate_bridge ;;
  reset) reset_env ;;
  *) echo "usage: source tools/dev_env.sh [pyvm|bridge|reset]" >&2 ;;
esac

