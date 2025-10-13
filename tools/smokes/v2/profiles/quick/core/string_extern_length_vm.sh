#!/bin/bash
# Ensure builtin String methods lower to nyrt.string externs with plugins OFF (legacy length())

# Skip by default in favor of size(); enable explicitly with SMOKES_ENABLE_LEGACY=1
if [ "${SMOKES_ENABLE_LEGACY:-0}" != "1" ]; then
  echo "SKIP: legacy length() test (enable SMOKES_ENABLE_LEGACY=1)" >&2
  exit 0
fi

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
