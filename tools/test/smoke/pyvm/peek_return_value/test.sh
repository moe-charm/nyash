#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release
mkdir -p "$ROOT/tmp"
emit_json "$ROOT/apps/tests/peek_return_value.nyash" "$ROOT/tmp/pyvm_peek_ret.json"
out=$(run_pyvm_json "$ROOT/tmp/pyvm_peek_ret.json")
echo "$out" | assert_grep '^1$'

