#!/bin/bash
# stage2_on_suite.sh — Convenience runner for Stage‑2 HostHandle Array smokes

set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
export NYASH_ROOT="${NYASH_ROOT:-$ROOT_DIR}"

# Enable plugins and Stage‑2 HostHandle array path
export NYASH_DISABLE_PLUGINS=0
export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
export HAKO_EXPORT_HOST=${HAKO_EXPORT_HOST:-1}

"$ROOT_DIR/tools/smokes/v2/profiles/plugins/map_keys_order_stage2_vm.sh" || true
"$ROOT_DIR/tools/smokes/v2/profiles/plugins/map_values_handle_mutation_vm.sh" || true
"$ROOT_DIR/tools/smokes/v2/profiles/plugins/map_array_handle_identity_vm.sh" || true

echo "[stage2-on-suite] Completed"

