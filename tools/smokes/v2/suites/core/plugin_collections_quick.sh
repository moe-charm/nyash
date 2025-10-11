#!/bin/bash
# Suite: core/plugin collections quick check (Array/Map/String via plugins)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../../../.." && pwd)"

cd "$ROOT"
SMOKES_FORCE_CONFIG=rust_vm_dynamic SMOKES_PROFILE_ENV=quick-core-plugins \
  tools/smokes/v2/run.sh --profile quick --filter 'plugin_on_*'
