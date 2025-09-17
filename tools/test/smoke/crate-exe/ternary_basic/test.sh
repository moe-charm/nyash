#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release; build_ny_llvmc; build_nyrt
mkdir -p "$ROOT/tmp"
emit_json "$ROOT/apps/tests/ternary_basic.nyash" "$ROOT/tmp/tb.json"
build_exe_crate "$ROOT/tmp/tb.json" "$ROOT/tmp/tb"
assert_exit "$ROOT/tmp/tb" 10

