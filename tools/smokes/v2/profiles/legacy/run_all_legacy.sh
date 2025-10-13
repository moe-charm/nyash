#!/bin/bash
# Run all legacy (transition) smokes. Not included in default profiles.

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"

bash "$DIR/legacy_map_keys_values_fallback_vm.sh" || true
bash "$DIR/legacy_array_size_force_host_vm.sh" || true
bash "$DIR/legacy_map_keys_values_bridge_vm.sh" || true
bash "$DIR/legacy_string_length_vm.sh" || true

echo "[legacy] Completed"
