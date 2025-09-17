#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release; build_ny_llvmc; build_nyrt
mkdir -p "$ROOT/tmp"
emit_json "$ROOT/apps/tests/peek_expr_block.nyash" "$ROOT/tmp/pb.json"
build_exe_crate "$ROOT/tmp/pb.json" "$ROOT/tmp/pb"
assert_exit "$ROOT/tmp/pb" 1

