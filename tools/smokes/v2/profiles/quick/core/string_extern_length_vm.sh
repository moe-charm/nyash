#!/bin/bash
# Ensure builtin String methods lower to nyrt.string externs with plugins OFF

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=1
export HAKO_PLUGIN_POLICY=off
export SMOKES_USE_PYVM=0
export NYASH_NYRT_SILENT_RESULT=0
require_env || exit 2

tmpfile=$(mktemp /tmp/string_extern_length_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "hello world"
    if s.length() != 11 { return 101 }
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
