#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release; build_ny_llvmc; build_nyrt
mkdir -p "$ROOT/tmp"
emit_json "$ROOT/apps/tests/ternary_nested.nyash" "$ROOT/tmp/tn.json"
build_exe_crate "$ROOT/tmp/tn.json" "$ROOT/tmp/tn"
assert_exit "$ROOT/tmp/tn" 50

