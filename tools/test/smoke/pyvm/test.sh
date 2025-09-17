#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

require_cmd python3
build_nyash_release

TMP_DIR=$(mktemp -d)
JSON="$TMP_DIR/ternary_basic.json"

APP="$ROOT/apps/tests/ternary_basic.nyash"
emit_json "$APP" "$JSON"

# Expect exit code 10 for ternary_basic
assert_exit "run_pyvm_json $JSON >/dev/null" 10
echo "OK: pyvm ternary_basic exit=10"
