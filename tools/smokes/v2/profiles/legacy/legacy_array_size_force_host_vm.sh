#!/bin/bash
# Wrapper: legacy host-slot forced Array.size path (transition aid)

set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
"$ROOT_DIR/tools/smokes/v2/profiles/quick/selfhost/host_handle_router_array_len_vm.sh"

