#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release
assert_exit "timeout -s KILL 60s bash $ROOT/tools/ny_stage2_shortcircuit_smoke.sh" 0

