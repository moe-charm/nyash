#!/usr/bin/env bash
# opbox-quick.sh — Quick profile with Operator Boxes enabled (single command)
# Usage: ./tools/opbox-quick.sh
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

source tools/dev_env.sh opbox

# Lighten preflight and set generous timeout
export SMOKES_PROVIDER_VERIFY_MODE=warn
export NYASH_DISABLE_PLUGINS=1
export SMOKES_DEFAULT_TIMEOUT=${SMOKES_DEFAULT_TIMEOUT:-180}

exec tools/smokes/v2/run.sh --profile quick --timeout "${SMOKES_DEFAULT_TIMEOUT}"

