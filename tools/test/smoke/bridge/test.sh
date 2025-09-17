#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release

# Use existing short-circuit smoke (ensures RHS not executed)
assert_exit "bash $ROOT/tools/ny_stage2_shortcircuit_smoke.sh >/dev/null" 0
echo "OK: bridge shortcircuit smoke"
