#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)

if ! command -v python3 >/dev/null 2>&1; then
  echo "[SKIP] python unit: python3 not available"; exit 0
fi

"$ROOT/tools/python_unit.sh" >/dev/null
echo "OK: python unit (phi_wiring helpers)"

