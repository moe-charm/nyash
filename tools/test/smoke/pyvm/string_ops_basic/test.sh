#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release
mkdir -p "$ROOT/tmp"
emit_json "$ROOT/apps/tests/string_ops_basic.nyash" "$ROOT/tmp/pyvm_string_ops.json"
out=$(run_pyvm_json "$ROOT/tmp/pyvm_string_ops.json")
echo "$out" | assert_grep '^len=5$'
echo "$out" | assert_grep '^sub=bcd$'
echo "$out" | assert_grep '^idx=1$'

