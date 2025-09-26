#!/usr/bin/env bash
# opbox-json.sh — Minimal JSON smoke runner with Operator Boxes enabled
# Usage: ./tools/opbox-json.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

# Enable Operator Boxes profile (dev)
source tools/dev_env.sh opbox

# Keep preflight light and avoid plugin stalls
export SMOKES_PROVIDER_VERIFY_MODE=warn
export NYASH_DISABLE_PLUGINS=1
export SMOKES_DEFAULT_TIMEOUT=${SMOKES_DEFAULT_TIMEOUT:-180}

echo "[opbox-json] Running JSON VM smokes (roundtrip + nested)"
tools/smokes/v2/profiles/quick/core/json_roundtrip_vm.sh
tools/smokes/v2/profiles/quick/core/json_nested_vm.sh

echo "[opbox-json] Done"
