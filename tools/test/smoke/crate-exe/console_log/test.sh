#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release
build_ny_llvmc
build_nyrt

TMP_DIR=$(mktemp -d)
SRC="$TMP_DIR/console_log_smoke.nyash"
JSON="$TMP_DIR/console_log_smoke.json"
EXE="$TMP_DIR/console_log_smoke.out"

cat >"$SRC" <<'NY'
static box Main {
  main() {
    print("hello-console")
    return 0
  }
}
NY

emit_json "$SRC" "$JSON"
build_exe_crate "$JSON" "$EXE"

assert_exit "$EXE" 0
echo "OK: crate-exe console.log smoke (exit=0)"
