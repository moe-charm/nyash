#!/bin/bash
# Ensure builtin String methods lower to nyrt.string externs (size)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=1
export HAKO_PLUGIN_POLICY=off
export SMOKES_USE_PYVM=0
export NYASH_NYRT_SILENT_RESULT=0
require_env || exit 2

tmpfile=$(mktemp /tmp/string_extern_size_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "hello world"
    if s.size() != 11 { return 101 }
    return 0
  }
}
SRC

if run_nyash_vm "$tmpfile" >/dev/null; then
  rm -f "$tmpfile"; echo "OK"; exit 0
else
  rc=$?; rm -f "$tmpfile"; echo "FAIL: rc=$rc" >&2; exit 1
fi

