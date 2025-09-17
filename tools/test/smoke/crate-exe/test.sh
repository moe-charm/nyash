#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

# Build binaries needed
build_nyash_release
build_ny_llvmc
build_nyrt

TMP_DIR=$(mktemp -d)
SRC="$TMP_DIR/crate_exe_smoke.nyash"
JSON="$TMP_DIR/crate_exe_smoke.json"
EXE="$TMP_DIR/crate_exe_smoke.out"

cat >"$SRC" <<'NY'
// minimal program returning 7 (no println to avoid unresolved symbols)
static box Main {
  main() {
    return 7
  }
}
NY

# Emit MIR JSON and build exe via crate compiler
emit_json "$SRC" "$JSON"
build_exe_crate "$JSON" "$EXE"

# Run and assert (exit code only)
set +e
OUT=$("$EXE" 2>&1)
CODE=$?
set -e
[[ "$CODE" -eq 7 ]] || { echo "exit=$CODE"; exit 1; }
echo "OK: crate-exe smoke (exit=7)"
