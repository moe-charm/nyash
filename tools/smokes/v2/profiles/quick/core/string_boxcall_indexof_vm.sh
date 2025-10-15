#!/bin/bash
# string_boxcall_indexof_vm.sh — StringBox.indexOf via BoxCall path (plugins OFF)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

tmpfile=$(mktemp /tmp/string_boxcall_indexof_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "Hello, Nyash!"
    // Expect indexOf to return 7 for "Ny"
    local i = s.indexOf("Ny")
    if i != 7 { return 101 }
    return 0
  }
}
SRC

run_nyash_vm "$tmpfile" >/dev/null
rc=$?
rm -f "$tmpfile"
if [ $rc -ne 0 ]; then
  echo "FAIL: rc=$rc" >&2
  exit 1
fi
echo "OK"
exit 0

