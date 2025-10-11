#!/bin/bash
# string_boxcall_substring_vm.sh — StringBox.substring via BoxCall path (plugins OFF)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

tmpfile=$(mktemp /tmp/string_boxcall_substring_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "Nyash"
    local t = s.substring(1, 3) // "ya"
    if t != "ya" { return 101 }
    return 0
  }
}
SRC

out_vm=$(run_nyash_vm "$tmpfile" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs)
rm -f "$tmpfile"

if [ $rc -ne 0 ]; then
  echo "FAIL: rc=$rc" >&2
  exit 1
fi
echo "OK"
exit 0

