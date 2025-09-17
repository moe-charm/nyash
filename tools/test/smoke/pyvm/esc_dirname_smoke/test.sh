#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release
mkdir -p "$ROOT/tmp"
emit_json "$ROOT/apps/tests/esc_dirname_smoke.nyash" "$ROOT/tmp/pyvm_esc_dir.json"
out=$(run_pyvm_json "$ROOT/tmp/pyvm_esc_dir.json")
# Expect two lines: escaped string and dirname join
echo "$out" | sed -n '1p' | assert_grep '^A\\\\\\"B\\\\\\\\C$'
echo "$out" | sed -n '2p' | assert_grep '^dir1/dir2$'

