#!/bin/bash
# Wrapper: legacy string-shim fallback for Map.keys()/values()

set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
"$ROOT_DIR/tools/smokes/v2/profiles/plugins/map_keys_values_fallback_vm.sh"

