#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release
mkdir -p "$ROOT/tmp"
emit_json "$ROOT/apps/tests/me_method_call.nyash" "$ROOT/tmp/pyvm_me_method.json"
out=$(run_pyvm_json "$ROOT/tmp/pyvm_me_method.json")
echo "$out" | assert_grep '^n=3$'

