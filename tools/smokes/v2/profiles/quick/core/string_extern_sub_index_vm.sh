#!/bin/bash
# Ensure substring/indexOf follow extern path with plugins OFF

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=1
export HAKO_PLUGIN_POLICY=off
export SMOKES_USE_PYVM=0
export NYASH_NYRT_SILENT_RESULT=0
require_env || exit 2

tmpfile=$(mktemp /tmp/string_extern_sub_index_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "hello world"
    if s.indexOf("world") != 6 { return 201 }
    if s.substring(0,5) != "hello" { return 202 }
    if s.charAt(1) != "e" { return 203 }
    return 0
  }
}
SRC

if run_nyash_vm "$tmpfile" >/dev/null; then
  rm -f "$tmpfile"
  echo "OK"
  exit 0
else
  rc=$?
  rm -f "$tmpfile"
  echo "FAIL: rc=$rc" >&2
  exit 1
fi
