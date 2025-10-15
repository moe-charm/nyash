#!/bin/bash
# Wrapper: legacy HostBridge wiring check for keysS/valuesS adapter path

set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
"$ROOT_DIR/tools/smokes/v2/profiles/quick-selfhost/map_keys_values_bridge_vm.sh"

